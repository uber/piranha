/**
 *    Copyright (c) 2019 Uber Technologies, Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

#include "clang/AST/AST.h"
#include "clang/AST/ASTConsumer.h"
#include "clang/AST/RecursiveASTVisitor.h"
#include "clang/ASTMatchers/ASTMatchFinder.h"
#include "clang/ASTMatchers/ASTMatchers.h"
#include "clang/Basic/Diagnostic.h"
#include "clang/Basic/SourceLocation.h"
#include "clang/Frontend/CompilerInstance.h"
#include "clang/Frontend/FrontendPluginRegistry.h"
#include "clang/Lex/Lexer.h"
#include "clang/Rewrite/Core/Rewriter.h"
#include "clang/Rewrite/Frontend/FixItRewriter.h"
#include "clang/Rewrite/Frontend/Rewriters.h"
#include "clang/Tooling/CommonOptionsParser.h"
#include "clang/Tooling/Tooling.h"
#include "llvm/ADT/APInt.h"

using namespace clang;
using namespace std;
using namespace clang::ast_matchers;
using namespace clang::driver;
using namespace clang::tooling;

// TODO: these strings corresponds to the flag APIs and are hard coded.
// Will need to be moved to a separate file subsequently to take in as separate
// parameters.
const string TREATED_METHOD_NAME = "isTreatedForExperiment:";
const string CONTROL_METHOD_NAME = "isInControlGroupForExperiment:";
const string OPTIMISTIC_METHOD_NAME =
    "optimisticFeatureFlagEnabledForExperiment:";
const string OPTIMISTIC_IS_ENABLED_METHOD_NAME = "isEnabled:";
const string INCLUSION_METHOD_NAME = "sendInclusionEventForExperiment:";
const string INCLUSION_TREATMENT_GROUP_METHOD_NAME =
    "sendInclusionEventForExperiment:forTreatmentGroup:";
const string ENABLED_FF_NAMED = "enableFeatureFlagNamed:";
const string DISABLED_EXP_NAMED = "disableExperimentNamed:";

enum Value { IS_TRUE, IS_FALSE, IS_UNKNOWN };
enum FlagType { UNDEFINED, CONTROL, TREATED, OPTIMISTIC };

namespace XPFlagRefactoring {
class FixItRewriterOptions : public FixItOptions {
public:
  string RewriteFilename(const string &Filename, int &fd) override {
    return Filename;
  }

  static void rewriteCode(ASTContext *context, FixItRewriter *rewriter,
                          SourceRange srcRange, SourceLocation srcLoc,
                          string replText) {
    const auto &FixIt = FixItHint::CreateReplacement(srcRange, replText);
    auto &DiagnosticsEngine = context->getDiagnostics();
    const auto ID = DiagnosticsEngine.getCustomDiagID(
        DiagnosticsEngine::Warning, "Refactoring stale flag related code");

    DiagnosticsEngine.Report(srcLoc, ID).AddFixItHint(FixIt);

    assert(rewriter != NULL);
    rewriter->WriteFixedFiles();
  }
};

class XPUtils {
public:
  // use the standard findSemiAfterLocation.
  static SourceLocation findSemiAfterLocation(SourceLocation loc,
                                              ASTContext &Ctx, bool IsDecl) {
    SourceManager &SM = Ctx.getSourceManager();
    if (loc.isMacroID()) {
      if (!Lexer::isAtEndOfMacroExpansion(loc, SM, Ctx.getLangOpts(), &loc))
        return SourceLocation();
    }

    loc = Lexer::getLocForEndOfToken(loc, 0, SM, Ctx.getLangOpts());

    // Break down the source location.
    std::pair<FileID, unsigned> locInfo = SM.getDecomposedLoc(loc);

    // Try to load the file buffer.
    bool invalidTemp = false;
    StringRef file = SM.getBufferData(locInfo.first, &invalidTemp);
    if (invalidTemp) {
      return SourceLocation();
    }
    const char *tokenBegin = file.data() + locInfo.second;

    // Lex from the start of the given location.
    Lexer lexer(SM.getLocForStartOfFile(locInfo.first), Ctx.getLangOpts(),
                file.begin(), tokenBegin, file.end());
    Token tok;
    lexer.LexFromRawLexer(tok);

    if (tok.isNot(tok::semi)) {
      if (!IsDecl)
        return SourceLocation();
      // Declaration may be followed with other tokens; such as an __attribute,
      // before ending with a semicolon.
      return findSemiAfterLocation(tok.getLocation(), Ctx, true);
    }

    return tok.getLocation();
  }

  static SourceRange getRequiredSourceRange(const Expr *s, ASTContext *context,
                                            char delimiter) {
    if (delimiter == ',' || delimiter == ';') {
      SourceManager *sm = &context->getSourceManager();
      LangOptions lopt = context->getLangOpts();

      SourceLocation b(s->getBeginLoc()), _e(s->getEndLoc());
      SourceLocation e(Lexer::getLocForEndOfToken(_e, 0, *sm, lopt));

      SourceRange *sr;

      if (delimiter == ',') {
        return SourceRange(b, e);
      }
      // search for a semicolon
      SourceLocation eSemi =
          findSemiAfterLocation(s->getEndLoc(), *context, true);
      sr =
          eSemi.isInvalid() ? new SourceRange(b, e) : new SourceRange(b, eSemi);
      return *sr;
    }
    return s->getSourceRange();
  }

  static string getExprSourceText(const Expr *s, ASTContext *context) {
    SourceManager *sm = &context->getSourceManager();
    LangOptions lopt = context->getLangOpts();

    SourceLocation b(s->getBeginLoc()), _e(s->getEndLoc());
    SourceLocation e(Lexer::getLocForEndOfToken(_e, 0, *sm, lopt));
    return std::string(sm->getCharacterData(b),
                       sm->getCharacterData(e) - sm->getCharacterData(b));
  }

  static FlagType getFlagType(const ObjCMessageExpr *msgExpr) {
    if (msgExpr != NULL) {
      const string currSelector = msgExpr->getSelector().getAsString();
      if (!currSelector.compare(TREATED_METHOD_NAME)) {
        return TREATED;
      } else if (!currSelector.compare(CONTROL_METHOD_NAME)) {
        return CONTROL;
      } else if (!currSelector.compare(OPTIMISTIC_METHOD_NAME)) {
        return OPTIMISTIC;
      }
    }
    return UNDEFINED;
  }

  static Value computeValue(FlagType foundType, FlagType flagType) {
    if ((flagType == OPTIMISTIC) ||
        (flagType == TREATED && foundType == TREATED) ||
        (flagType == CONTROL && foundType == CONTROL)) {
      return IS_TRUE;
    } else {
      return IS_FALSE;
    }
  }

  static Value evalExpr(const Expr *expr,
                        const MatchFinder::MatchResult &Result,
                        const ObjCMessageExpr *reqSelector, FlagType flagType) {

    if (isa<ParenExpr>(expr)) {
      const ParenExpr *pe = cast<ParenExpr>(expr);
      const Expr *eSub = pe->getSubExpr();
      if (eSub != NULL) {
        return evalExpr(eSub, Result, reqSelector, flagType);
      }
    }

    if (isa<IntegerLiteral>(expr)) {
      const IntegerLiteral *il = cast<IntegerLiteral>(expr);
      if (!(il->getValue())) {
        return IS_FALSE;
      } else {
        return IS_TRUE;
      }
    }

    if (isa<UnaryOperator>(expr)) {
      const UnaryOperator *uo = cast<UnaryOperator>(expr);
      const Expr *eSub = uo->getSubExpr();
      if (eSub != NULL) {
        Value v = evalExpr(eSub, Result, reqSelector, flagType);
        if (uo->getOpcode() == UO_LNot) {
          if (v == IS_TRUE) {
            return IS_FALSE;
          } else if (v == IS_FALSE) {
            return IS_TRUE;
          }
          return IS_UNKNOWN;
        } else {
          return v;
        }
      }
    }

    if (isa<BinaryOperator>(expr)) {
      const BinaryOperator *bo = cast<BinaryOperator>(expr);
      const Expr *lhs = bo->getLHS();
      const Expr *rhs = bo->getRHS();

      Value l = evalExpr(lhs, Result, reqSelector, flagType);
      Value r = evalExpr(rhs, Result, reqSelector, flagType);

      if (bo->getOpcode() == BO_LOr) {
        if (l == IS_TRUE || r == IS_TRUE) {
          return IS_TRUE;
        }
        if (l == IS_FALSE && r == IS_FALSE) {
          return IS_FALSE;
        }
      } else if (bo->getOpcode() == BO_LAnd) {
        if (l == IS_TRUE && r == IS_TRUE) {
          return IS_TRUE;
        }
        if (l == IS_FALSE || r == IS_FALSE) {
          return IS_FALSE;
        }
      }
    }

    if (isa<ObjCMessageExpr>(expr)) {
      const ObjCMessageExpr *msgExpr = cast<ObjCMessageExpr>(expr);
      if (msgExpr != NULL && (msgExpr == reqSelector)) {
        FlagType foundType = getFlagType(msgExpr);
        Value v = computeValue(foundType, flagType);
        return v;
      } else {
        return IS_UNKNOWN;
      }
    }

    if (isa<ExprWithCleanups>(expr)) {
      const ExprWithCleanups *ewc = cast<ExprWithCleanups>(expr);
      const Expr *eSub = ewc->getSubExpr();
      if (eSub != NULL) {
        return evalExpr(eSub, Result, reqSelector, flagType);
      }
    }

    if (isa<ImplicitCastExpr>(expr)) {
      const ImplicitCastExpr *ice = cast<ImplicitCastExpr>(expr);
      const Expr *eSub = ice->getSubExpr();
      if (eSub != NULL) {
        return evalExpr(eSub, Result, reqSelector, flagType);
      }
    }

    if (isa<PseudoObjectExpr>(expr)) {
      const PseudoObjectExpr *poe = cast<PseudoObjectExpr>(expr);
      const Expr *eResult = poe->getResultExpr();
      if (eResult != NULL) {
        return evalExpr(eResult, Result, reqSelector, flagType);
      }
    }

    return IS_UNKNOWN;
  }
};

class FlagDeclarationHandler : public MatchFinder::MatchCallback {
public:
  using MatchResult = MatchFinder::MatchResult;
  using RewriterPointer = unique_ptr<FixItRewriter>;

  FlagDeclarationHandler(string flagName) : flagName(flagName) {}

  virtual void run(const MatchFinder::MatchResult &Result) {
    const DeclRefExpr *dre = Result.Nodes.getNodeAs<DeclRefExpr>("theDecl");
    if (dre != NULL) {
      ASTContext *context = Result.Context;
      SourceRange sr = XPUtils::getRequiredSourceRange(dre, context, ',');
      llvm::errs() << "Rewriting in FlagDeclarationHandler \n";
      FixItRewriterOptions::rewriteCode(context, rewriter, sr,
                                        dre->getBeginLoc(), "");
      return;
    }
  }

  void setWriter(FixItRewriter *rewriter) { this->rewriter = rewriter; }

private:
  string flagName;
  FixItRewriter *rewriter;
};

class MethodInvocationHandler : public MatchFinder::MatchCallback {
public:
  using MatchResult = MatchFinder::MatchResult;
  using RewriterPointer = unique_ptr<FixItRewriter>;

  MethodInvocationHandler(string flagName) : flagName(flagName) {}

  virtual void run(const MatchFinder::MatchResult &Result) {
    const ObjCMessageExpr *msgExpr =
        Result.Nodes.getNodeAs<ObjCMessageExpr>("theSelector");

    if (msgExpr != NULL) {
      ASTContext *context = Result.Context;
      SourceRange sr = XPUtils::getRequiredSourceRange(msgExpr, context, ';');
      llvm::errs() << "Rewriting in MethodInvocationHandler \n";

      FixItRewriterOptions::rewriteCode(context, rewriter, sr,
                                        msgExpr->getBeginLoc(), "");
      return;
    }
  }

  void setWriter(FixItRewriter *rewriter) { this->rewriter = rewriter; }

private:
  string flagName;
  FixItRewriter *rewriter;
};

class MethodImplHandler : public MatchFinder::MatchCallback {
public:
  using MatchResult = MatchFinder::MatchResult;
  using RewriterPointer = unique_ptr<FixItRewriter>;

  MethodImplHandler(string flagName) : flagName(flagName) {}

  virtual void run(const MatchFinder::MatchResult &Result) {
    const ObjCMethodDecl *method =
        Result.Nodes.getNodeAs<ObjCMethodDecl>("theMethod");
    if (method != NULL) {
      ASTContext *context = Result.Context;
      llvm::errs() << "Rewriting in MethodImplHandler \n";
      FixItRewriterOptions::rewriteCode(context, rewriter,
                                        method->getSourceRange(),
                                        method->getBeginLoc(), "");
      return;
    }
  }

  void setWriter(FixItRewriter *rewriter) { this->rewriter = rewriter; }

private:
  string flagName;
  FixItRewriter *rewriter;
};

class BinOpHandler : public MatchFinder::MatchCallback {

public:
  using MatchResult = MatchFinder::MatchResult;
  using RewriterPointer = unique_ptr<FixItRewriter>;

  BinOpHandler(string flagName, FlagType flagType)
      : flagName(flagName), flagType(flagType) {}

  virtual void run(const MatchFinder::MatchResult &Result) {
    const BinaryOperator *s = Result.Nodes.getNodeAs<BinaryOperator>("binOp");
    const ObjCMessageExpr *msgExpr =
        Result.Nodes.getNodeAs<ObjCMessageExpr>("theSelector");
    const StringLiteral *strLiteral =
        Result.Nodes.getNodeAs<StringLiteral>("theFlag");
    ASTContext *context = Result.Context;

    // avoid double processing of the same selector.
    if (msgExpr != NULL && processedExprs.find(msgExpr) != processedExprs.end())
      return;

    // ensure that the string literal, if it exists, matches the flagName.
    if (strLiteral != NULL && strLiteral->getString().compare(flagName)) {
      return;
    }

    Value v = IS_UNKNOWN;
    if (s != NULL) {
      v = XPUtils::evalExpr(s, Result, msgExpr, flagType);
    }

    string replacementText = "";
    Value l = XPUtils::evalExpr(s->getLHS(), Result, msgExpr, flagType);
    Value r = XPUtils::evalExpr(s->getRHS(), Result, msgExpr, flagType);

    const Expr *nodeToReplace = s;
    char delimiter = '\0';

    if (v == IS_UNKNOWN) {
      if (s->getOpcode() == BO_LAnd) {
        if (l == IS_TRUE) {
          replacementText = XPUtils::getExprSourceText(s->getRHS(), context);
        } else if (r == IS_TRUE) {
          replacementText = XPUtils::getExprSourceText(s->getLHS(), context);
        }
      } else if (s->getOpcode() == BO_LOr) {
        if (l == IS_FALSE) {
          replacementText = XPUtils::getExprSourceText(s->getRHS(), context);
        } else if (r == IS_FALSE) {
          replacementText = XPUtils::getExprSourceText(s->getLHS(), context);
        }
      } else {
        if (s->getOpcode() == BO_Assign) {
          nodeToReplace = s->getRHS();
          delimiter = ';';
          if (r == IS_TRUE) {
            replacementText = "true;";
          } else if (r == IS_FALSE) {
            replacementText = "false;";
          }
        }
      }
    } else if (v == IS_TRUE) {
      replacementText = "true";
    } else if (v == IS_FALSE) {
      replacementText = "false";
    }

    if (replacementText.compare("")) {
      processedExprs.insert(make_pair(msgExpr, true));
      SourceRange sr =
          XPUtils::getRequiredSourceRange(nodeToReplace, context, delimiter);
      llvm::errs() << "Rewriting in BinOpHandler \n";
      FixItRewriterOptions::rewriteCode(
          context, rewriter, sr, nodeToReplace->getBeginLoc(), replacementText);
    }
  }

  void setWriter(FixItRewriter *rewriter) { this->rewriter = rewriter; }

private:
  string flagName;
  FlagType flagType;
  FixItRewriter *rewriter;
  map<const ObjCMessageExpr *, bool> processedExprs;
};

class ConditionalOpHandler : public MatchFinder::MatchCallback {
public:
  using MatchResult = MatchFinder::MatchResult;
  using RewriterPointer = unique_ptr<FixItRewriter>;

  ConditionalOpHandler(string flagName, FlagType flagType)
      : flagName(flagName), flagType(flagType) {}

  virtual void run(const MatchFinder::MatchResult &Result) {
    const ObjCMessageExpr *msgExpr =
        Result.Nodes.getNodeAs<ObjCMessageExpr>("theSelector");
    ASTContext *context = Result.Context;

    const Expr *conditionExpr = Result.Nodes.getNodeAs<Expr>("conditionExpr");

    Value v = IS_UNKNOWN;

    if (conditionExpr != NULL) {
      v = XPUtils::evalExpr(conditionExpr, Result, msgExpr, flagType);
    }

    if (v == IS_UNKNOWN) {
      return;
    }

    const Stmt *s = Result.Nodes.getNodeAs<ConditionalOperator>("condStmt");
    if (isa<ConditionalOperator>(s)) {
      const ConditionalOperator *co = cast<ConditionalOperator>(s);
      string replacementText = "";
      if (v == IS_TRUE) {
        replacementText =
            XPUtils::getExprSourceText(co->getTrueExpr(), context);
      } else {
        replacementText =
            XPUtils::getExprSourceText(co->getFalseExpr(), context);
      }
      llvm::errs() << "Rewriting in ConditionalOpHandler \n";
      FixItRewriterOptions::rewriteCode(context, rewriter, co->getSourceRange(),
                                        co->getBeginLoc(), replacementText);
    }
  }

  void setWriter(FixItRewriter *rewriter) { this->rewriter = rewriter; }

private:
  string flagName;
  FlagType flagType;
  FixItRewriter *rewriter;
};

class IfStmtHandler : public MatchFinder::MatchCallback {
public:
  using MatchResult = MatchFinder::MatchResult;
  using RewriterPointer = unique_ptr<FixItRewriter>;

  IfStmtHandler(string flagName, FlagType flagType)
      : flagName(flagName), flagType(flagType) {}

  virtual void run(const MatchFinder::MatchResult &Result) {

    const IfStmt *s = Result.Nodes.getNodeAs<IfStmt>("ifStmt");
    const ObjCMessageExpr *msgExpr =
        Result.Nodes.getNodeAs<ObjCMessageExpr>("theSelector");
    const StringLiteral *strLiteral =
        Result.Nodes.getNodeAs<StringLiteral>("theFlag");
    ASTContext *context = Result.Context;

    // ensure that the string literal, if it exists, matches the flagName.
    if (strLiteral != NULL && strLiteral->getString().compare(flagName)) {
      return;
    }

    const Expr *conditionExpr = Result.Nodes.getNodeAs<Expr>("conditionExpr");
    Value v = IS_UNKNOWN;
    if (conditionExpr != NULL) {
      v = XPUtils::evalExpr(conditionExpr, Result, msgExpr, flagType);
    }

    if (v == IS_UNKNOWN) {
      return;
    }

    string replacementText = "";
    if (v == IS_TRUE) {
      replacementText = getReplText(s->getThen(), context);
    } else {
      const Stmt *Else = s->getElse();
      if (Else) {
        replacementText = getReplText(Else, context);
      }
    }
    llvm::errs() << "Rewriting in IfStmtHandler \n";
    FixItRewriterOptions::rewriteCode(context, rewriter, s->getSourceRange(),
                                      s->getBeginLoc(), replacementText);
  }

  void setWriter(FixItRewriter *rewriter) { this->rewriter = rewriter; }

private:
  string flagName;
  FlagType flagType;
  FixItRewriter *rewriter;

  StringRef getReplText(const Stmt *s, ASTContext *context) {
    StringRef res = Lexer::getSourceText(
        CharSourceRange::getCharRange(s->getSourceRange()),
        context->getSourceManager(), context->getLangOpts());
    res = res.ltrim('{');
    res = res.ltrim('\n');
    res = res.rtrim(' ');
    res = res.rtrim('\n');
    return res;
  }
};

class XPRefactorConsumer : public ASTConsumer {
public:
  XPRefactorConsumer(CompilerInstance &CI, string flagName, FlagType flagType,
                     bool shouldHandleMethodImpl)
      : HandlerForIf(flagName, flagType), HandlerForMethodImpl(flagName),
        HandlerForMethodInvocation(flagName), HandlerForDeclRef(flagName),
        HandlerForBinOp(flagName, flagType),
        HandlerForConditionalOp(flagName, flagType) {

   // ignore missing headers
   if(CI.hasPreprocessor()) {
     CI.getPreprocessor().SetSuppressIncludeNotFoundError(true);
   }

    // matches if([self.....<flagName>])
    Matcher.addMatcher(
        ifStmt(
            hasCondition(
                exprWithCleanups(
                    hasDescendant(objcMessageExpr(
                                      anyOf(hasSelector(TREATED_METHOD_NAME),
                                            hasSelector(CONTROL_METHOD_NAME)),
                                      hasArgument(0, declRefExpr(to(varDecl(
                                                         hasName(flagName))))))
                                      .bind("theSelector")))
                    .bind("conditionExpr")))
            .bind("ifStmt"),
        &HandlerForIf);

    // updated AST handler matches if(optimisticallyTreated..(<flagName>)
    Matcher.addMatcher(
        ifStmt(hasCondition(
                   expr(hasDescendant(
                            objcMessageExpr(
                                hasSelector(OPTIMISTIC_IS_ENABLED_METHOD_NAME),
                                hasDescendant(stringLiteral().bind("theFlag")))
                                .bind("theSelector")))
                       .bind("conditionExpr")))
            .bind("ifStmt"),
        &HandlerForIf);

    // matches if(optimisticallyTreated..(<flagName>)
    Matcher.addMatcher(
        ifStmt(hasCondition(
                   exprWithCleanups(
                       hasDescendant(
                           objcMessageExpr(
                               hasSelector(OPTIMISTIC_METHOD_NAME),
                               hasDescendant(objcMessageExpr(
                                   hasDescendant(objcMessageExpr(hasDescendant(
                                       stringLiteral().bind("theFlag")))))))
                               .bind("theSelector")))
                       .bind("conditionExpr")))
            .bind("ifStmt"),
        &HandlerForIf);

    // matches if(<classname>.<flagName>)
    Matcher.addMatcher(
        ifStmt(
            hasCondition(
                expr(hasDescendant(objcMessageExpr(hasSelector(flagName))
                                       .bind("theSelector") // selectorIsFlag")
                                   ))
                    .bind("conditionExpr")))
            .bind("ifStmt"),
        &HandlerForIf);

    if (shouldHandleMethodImpl) {
      // matches implementation/definition of method with flagName
      Matcher.addMatcher(objcMethodDecl(hasName(flagName)).bind("theMethod"),
                         &HandlerForMethodImpl);
    }

    // matches [self sendInclusionEvent...:<flagName>]
    Matcher.addMatcher(
        objcMessageExpr(
            anyOf(hasSelector(INCLUSION_METHOD_NAME),
                  hasSelector(INCLUSION_TREATMENT_GROUP_METHOD_NAME),
                  hasSelector(ENABLED_FF_NAMED),
                  hasSelector(DISABLED_EXP_NAMED)),
            hasArgument(0, declRefExpr(to(varDecl(hasName(flagName))))))
            .bind("theSelector"),
        &HandlerForMethodInvocation);

    // matches if([self.....<flagName>])
    Matcher.addMatcher(
        binaryOperator(
            hasDescendant(
                objcMessageExpr(
                    anyOf(hasSelector(TREATED_METHOD_NAME),
                          hasSelector(CONTROL_METHOD_NAME)),
                    hasArgument(0, declRefExpr(to(varDecl(hasName(flagName))))))
                    .bind("theSelector")))
            .bind("binOp"),
        &HandlerForBinOp);

    Matcher.addMatcher(
        binaryOperator(
            hasDescendant(
                objcMessageExpr(hasSelector(flagName)).bind("theSelector")))
            .bind("binOp"),
        &HandlerForBinOp);

    // a hack as there doesn't seem to be a matcher for ObjCArrayLiteral
    Matcher.addMatcher(
        objcMessageExpr(
            hasDescendant(implicitCastExpr(
                hasDescendant(implicitCastExpr(hasDescendant(implicitCastExpr(
                    hasDescendant(declRefExpr(to(varDecl(hasName(flagName))))
                                      .bind("theDecl")))))))))
            .bind("objExpr"),
        &HandlerForDeclRef);

    // matches if(<classname>.<flagName>)
    Matcher.addMatcher(
        conditionalOperator(
            hasCondition(
                expr(hasDescendant(objcMessageExpr(hasSelector(flagName))
                                       .bind("theSelector")))
                    .bind("conditionExpr")))
            .bind("condStmt"),
        &HandlerForConditionalOp);

    //   Matcher.addMatcher(ifStmt().bind("ifStmt"), &HandlerForIf);
  }

  void HandleTranslationUnit(ASTContext &context) {
    auto &DE = context.getDiagnostics();
    FixItRewriterOptions FixItOptions;
    FixItOptions.InPlace = true;
    rewriter = new FixItRewriter(DE, context.getSourceManager(),
                                 context.getLangOpts(), &FixItOptions);
    DE.setClient(rewriter, false);

    HandlerForIf.setWriter(rewriter);
    HandlerForMethodImpl.setWriter(rewriter);
    HandlerForMethodInvocation.setWriter(rewriter);
    HandlerForDeclRef.setWriter(rewriter);

    HandlerForBinOp.setWriter(rewriter);
    HandlerForConditionalOp.setWriter(rewriter);
    Matcher.matchAST(context);
  }

private:
  ast_matchers::MatchFinder Matcher;
  IfStmtHandler HandlerForIf;
  MethodImplHandler HandlerForMethodImpl;
  MethodInvocationHandler HandlerForMethodInvocation;
  FlagDeclarationHandler HandlerForDeclRef;
  BinOpHandler HandlerForBinOp;
  ConditionalOpHandler HandlerForConditionalOp;
  FixItRewriter *rewriter;
};

class XPRefactorASTAction : public PluginASTAction {
public:
  virtual unique_ptr<ASTConsumer> CreateASTConsumer(CompilerInstance &CI,
                                                    llvm::StringRef InFile) {
    return unique_ptr<ASTConsumer>(
        new XPRefactorConsumer(CI, flagName, flagType, shouldHandleMethodImpl));
  }

  bool ParseArgs(const CompilerInstance &CI, const vector<string> &args) {
    // this should have only one argument
    assert(args.size() == 1);

    string s = args.at(0);
    auto comma = s.find(",");
    flagName = s.substr(0, comma);

    string remaining = s.substr(comma + 1, string::npos);

    comma = remaining.find(",");
    string strFlagType = remaining.substr(0, comma);

    if (!strFlagType.compare("control")) {
      flagType = CONTROL;
    } else if (!strFlagType.compare("treated")) {
      flagType = TREATED;
    } else if (!strFlagType.compare("optimistic")) {
      flagType = OPTIMISTIC;
    } else {
      flagType = UNDEFINED;
    }

    shouldHandleMethodImpl = true;
    if (comma != string::npos) {
      remaining = remaining.substr(comma + 1, string::npos);
      if (!remaining.compare("doNotHandleMethodImpl")) {
        shouldHandleMethodImpl = false;
      }
    }
    return true;
  }

private:
  string flagName;
  FlagType flagType;
  bool shouldHandleMethodImpl;
};
} // namespace XPFlagRefactoring

static clang::FrontendPluginRegistry::Add<
    XPFlagRefactoring::XPRefactorASTAction>
    X("XPFlagRefactorPlugin", "ObjectiveC - Stale XP Flag Refactoring");

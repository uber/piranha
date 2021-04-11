package com.uber.piranha.testannotations;

import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableCollection;
import com.google.common.collect.ImmutableMultimap;
import com.google.common.collect.ImmutableSet;
import com.google.errorprone.VisitorState;
import com.google.errorprone.util.ASTHelpers;
import com.sun.source.tree.AnnotationTree;
import com.sun.source.tree.AssignmentTree;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.LiteralTree;
import com.sun.source.tree.MethodTree;
import com.sun.source.tree.NewArrayTree;
import com.sun.source.tree.Tree;
import com.uber.piranha.config.PiranhaConfigurationException;
import java.util.HashSet;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;
import org.json.simple.JSONObject;

public final class TestAnnotationResolver {

  // We just need this to be an unique string which can never clash with a flag or group name, it's
  // used as a special value to indicate a field of an annotation is not currently of a parseable
  // type.
  private static final String UNPARSED_FIELD_VALUE =
      "$$$UNPARSED$$$9A1B0050-1BBD-420F-B6B3-AA0DB7D0AAE8";

  /**
   * testAnnotationSpecs is a map where key is the annotation name and the value is an object
   * encoding the specification of a testing annotations understood by Piranha, including how to
   * parse: a) the flag being tested, b) whether the test is requesting the flag in treatment or
   * control mode, c) (optionally) whether a particular treatment group is being specified
   */
  private final ImmutableMultimap<String, TestAnnotationSpecRecord> testAnnotationSpecs;

  private TestAnnotationResolver(
      ImmutableMultimap<String, TestAnnotationSpecRecord> testAnnotationSpecs) {
    this.testAnnotationSpecs = testAnnotationSpecs;
  }

  private static ImmutableMultimap<String, AnnotationArgument> parseAnnotationArguments(
      AnnotationTree at) {
    ImmutableMultimap.Builder<String, AnnotationArgument> builder = ImmutableMultimap.builder();
    for (ExpressionTree et : at.getArguments()) {
      if (et.getKind() == Tree.Kind.ASSIGNMENT) {
        AssignmentTree assn = (AssignmentTree) et;
        String key = "$" + ASTHelpers.getSymbol(assn.getVariable()).getSimpleName().toString();
        ExpressionTree assnExpression = assn.getExpression();
        Tree.Kind assnExprKind = assnExpression.getKind();
        switch (assnExprKind) {
          case IDENTIFIER: // Fallthrough
          case MEMBER_SELECT:
            builder.put(
                key,
                new AnnotationArgument(
                    ASTHelpers.getSymbol(assnExpression).getSimpleName().toString(),
                    assnExpression));
            break;
          case STRING_LITERAL: // Fallthrough
          case BOOLEAN_LITERAL:
            builder.put(
                key,
                new AnnotationArgument(
                    ((LiteralTree) assnExpression).getValue().toString(), assnExpression));
            break;
          case NEW_ARRAY:
            // For each in the array
            NewArrayTree arrayExpression = (NewArrayTree) assnExpression;
            for (ExpressionTree expr : arrayExpression.getInitializers()) {
              if (expr instanceof LiteralTree) {
                builder.put(
                    key, new AnnotationArgument(((LiteralTree) expr).getValue().toString(), expr));
              } else {
                builder.put(
                    key,
                    new AnnotationArgument(
                        ASTHelpers.getSymbol(expr).getSimpleName().toString(), expr));
              }
            }
            break;
          default:
            builder.put(key, new AnnotationArgument(UNPARSED_FIELD_VALUE, assnExpression));
        }
      }
    }
    return builder.build();
  }

  private Optional<ResolvedTestAnnotation> attemptResolveSpec(
      TestAnnotationSpecRecord spec, AnnotationTree at, Tree parent, VisitorState state) {
    if (!at.getAnnotationType().toString().contains(spec.getAnnotationName())) {
      // For sanity, attempting to resolve an annotation with a spec without the same name will
      // always return
      // empty.
      return Optional.empty();
    }
    ImmutableMultimap<String, AnnotationArgument> fields = parseAnnotationArguments(at);
    Set<String> relevantFieldParsingErrors = new HashSet<String>();
    // Process flag id(s)
    ImmutableSet.Builder<AnnotationArgument> flagIdsBuilder = ImmutableSet.builder();
    String flagIdentifierOrField = spec.getFlagIdentifierOrField();
    if (TestAnnotationSpecRecord.isFieldReference(flagIdentifierOrField)) {
      // Annotation field reference
      if (!fields.containsKey(flagIdentifierOrField)) {
        // Annotation does not match the spec. That's fine, maybe another spec with the same name
        // does.
        return Optional.empty();
      }
      for (AnnotationArgument argument : fields.get(flagIdentifierOrField)) {
        if (argument.getValue().equals(UNPARSED_FIELD_VALUE)) {
          relevantFieldParsingErrors.add(
              String.format(
                  "Field %s is not of a type which can be parsed to a flag or list of flags.",
                  flagIdentifierOrField));
        }
        flagIdsBuilder.add(argument);
      }
    } else {
      // constant flag name
      flagIdsBuilder.add(new AnnotationArgument(flagIdentifierOrField, null));
    }
    // Process flag treatment value
    boolean treated = false;
    String treatedValueOrField = spec.getTreatedValueOrField();
    if (TestAnnotationSpecRecord.isFieldReference(treatedValueOrField)) {
      if (!fields.containsKey(treatedValueOrField)) {
        // Annotation does not match the spec. That's fine, maybe another spec with the same name
        // does.
        return Optional.empty();
      } else if (fields.get(treatedValueOrField).size() != 1) {
        relevantFieldParsingErrors.add(
            String.format(
                "Field %s is not valid for specifying the treatment condition. Expected single "
                    + "boolean (or string encoding boolean) value.",
                treatedValueOrField));
      }
      AnnotationArgument argument = fields.get(treatedValueOrField).asList().get(0);
      if (argument.getValue().equals("true")) {
        treated = true;
      } else if (argument.getValue().equals("false")) {
        treated = false;
      } else {
        relevantFieldParsingErrors.add(
            String.format(
                "Field %s is not valid for specifying the treatment condition. Expected a "
                    + "boolean (or string encoding boolean) value.",
                treatedValueOrField));
      }
    } else {
      // Only 'true' or 'false' are possible at this point, due to parsing logic
      Preconditions.checkArgument(
          treatedValueOrField.equals("true") || treatedValueOrField.equals("false"));
      treated = treatedValueOrField.equals("true");
    }
    // Process treatment group, if any
    ImmutableSet.Builder<AnnotationArgument> groupsBuilder = ImmutableSet.builder();
    Optional<String> groupValueOrField = spec.getGroupValueOrField();
    if (groupValueOrField.isPresent()) {
      // ToDo: implement
      throw new UnsupportedOperationException(
          "Treatment group information in test annotations not yet supported");
    }
    if (!relevantFieldParsingErrors.isEmpty()) {
      throw new AnnotationResolutionException(
          String.format(
              "Found annotation %s at %s, matching the shape of the test annotation "
                  + "specification `%s` from Piranha's configuration file. However, one or more errors "
                  + "occurred when parsing the annotation's fields:%s",
              state.getSourceForNode(at),
              ASTHelpers.getSymbol(parent).flatName().toString(),
              spec,
              relevantFieldParsingErrors
                  .stream()
                  .map(a -> a.toString())
                  .collect(Collectors.joining("\n\t* "))));
    }
    return Optional.of(
        new ResolvedTestAnnotation(flagIdsBuilder.build(), treated, groupsBuilder.build(), at));
  }

  public ImmutableSet<ResolvedTestAnnotation> resolveAllForMethod(
      MethodTree tree, VisitorState state) {
    ImmutableSet.Builder<ResolvedTestAnnotation> builder = ImmutableSet.builder();
    for (String name : testAnnotationSpecs.keySet()) {
      AnnotationTree at =
          ASTHelpers.getAnnotationWithSimpleName(tree.getModifiers().getAnnotations(), name);

      if (at != null) {
        // Find all annotations specs with a matching name
        ImmutableCollection<TestAnnotationSpecRecord> candidateAnnotations =
            testAnnotationSpecs.get(name);
        boolean matchFound = false;
        for (TestAnnotationSpecRecord spec : candidateAnnotations) {
          Optional<ResolvedTestAnnotation> resolvedSpec = attemptResolveSpec(spec, at, tree, state);
          if (resolvedSpec.isPresent()) {
            matchFound = true;
            builder.add(resolvedSpec.get());
            break; // First match found is sufficient, assume specs of the same name are
                   // incompatible.
          }
        }
        if (!matchFound) {
          throw new AnnotationResolutionException(
              String.format(
                  "Found annotation %s at %s, matching the name of a test annotation "
                      + "declared in Piranha's config file. However, the annotation can't be matched "
                      + "to any of the specified shapes in this configuration. Candidates are:%s",
                  state.getSourceForNode(at),
                  ASTHelpers.getSymbol(tree).flatName().toString(),
                  candidateAnnotations
                      .stream()
                      .map(a -> a.toString())
                      .collect(Collectors.joining("\n\t"))));
        }
      }
    }
    return builder.build();
  }

  public static Builder builder() {
    return new Builder();
  }

  public static final class Builder {
    private final ImmutableMultimap.Builder<String, TestAnnotationSpecRecord> inner =
        ImmutableMultimap.builder();

    private Builder() {
      // Empty constructor
    }

    public void addFromName(String name) {
      inner.put(name, TestAnnotationSpecRecord.fromName(name));
    }

    public void addFromJSONObject(JSONObject jsonObject) throws PiranhaConfigurationException {
      TestAnnotationSpecRecord record = TestAnnotationSpecRecord.fromJSONObject(jsonObject);
      inner.put(record.getAnnotationName(), record);
    }

    public TestAnnotationResolver build() {
      return new TestAnnotationResolver(inner.build());
    }
  }
}

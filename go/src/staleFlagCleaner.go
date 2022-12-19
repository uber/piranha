/*
Copyright (c) 2021 Uber Technologies, Inc.
Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file

except in compliance with the License. You may obtain a copy of the License at
http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the

License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
express or implied. See the License for the specific language governing permissions and
limitations under the License.
*/
package src

import (
	"encoding/json"
	"fmt"
	"go/token"
	"io/ioutil"
	"os"

	/*
		We have used dst package instead of official go ast package because ast package
		does not keep positions of comments, we use dst package which keeps the positions
		of comment in place. It is similiar to the ast package and have same functions as ast.
		So you can use documentation of ast to see the details of helper functions.
	*/
	"github.com/dave/dst"
	"github.com/dave/dst/dstutil"
)

// API describes the type of API.
type API int

const (
	_isTreated API = iota
	_isControl
	_isTesting
	_isUnknown
)

// Methods describes the parent field of the json format.
type Methods struct {
	Methods []FlagAPI `json:"methodProperties"`
}

// FlagAPI denotes the field values in json format.
type FlagAPI struct {
	MethodName string `json:"methodName"`
	OfType     string `json:"flagType"`
	FlagIndex  int    `json:"argumentIndex"`
}

// Value is made to not confuse with true and false in code.
type Value int

const (
	isTrue Value = iota
	isFalse
	// This will be returned when you don't have to find true or false.
	isUndefined
)

// Some useful global variables
var (
	modeTreated = "treated"
	modeControl = "control"
)
var (
	strTrue  = "true"
	strFalse = "false"
)

// Defining a error class in here
type initError struct {
	errStr string
}

func (je *initError) Error() string {
	return fmt.Sprintf("staleFlagCleaner initialization error: %s", je.errStr)
}

// Implementing this as whole class
type staleFlagCleaner struct {
	configFile string
	methods    Methods

	/*
		Here valueMap contains those variable names whose value are affected by the stale function output.
		It means that this will used for cleaning in Deep Clean Pass.
	*/
	valueMap map[string]Value
	functype map[string]FlagAPI

	flagName  string
	isTreated bool
}

func (sfc *staleFlagCleaner) init(configFile string, flagName string, isTreated bool) error {
	sfc.configFile = configFile
	sfc.flagName = flagName
	sfc.isTreated = isTreated
	sfc.valueMap = make(map[string]Value)
	sfc.functype = make(map[string]FlagAPI)
	err := sfc.ParseJSON()
	return err
}

// Parse json
func (sfc *staleFlagCleaner) ParseJSON() error {

	jsonFile, err := os.Open(sfc.configFile)
	if err != nil {
		return &initError{"Failed opening the config file."}
	}

	byteValue, err := ioutil.ReadAll(jsonFile)
	if err != nil {
		return &initError{"Failed processing the config file, unable to read it."}
	}

	// we unmarshal our byteArray which contains our
	// jsonFile's content into 'methodProperties' which we defined above
	err = json.Unmarshal(byteValue, &sfc.methods)
	if err != nil {
		return &initError{"Failed processing the config file, unable to unmarshal json file."}
	}
	// store it in a hashmap
	for _, element := range sfc.methods.Methods {
		sfc.functype[element.MethodName] = element
	}
	return nil
}

// Gets the type of the flagAPI
func (sfc *staleFlagCleaner) flagTypeAPI(callExpr *dst.CallExpr) API {
	var apiName string

	switch d := callExpr.Fun.(type) {
	case *dst.Ident:
		apiName = d.Name
	case *dst.SelectorExpr:
		apiName = d.Sel.Name
	default:
		// other cases will come in future
	}

	if element, ok := sfc.functype[apiName]; ok {
		if element.FlagIndex < len(callExpr.Args) {
			switch arg := callExpr.Args[element.FlagIndex].(type) {
			case *dst.Ident:
				if arg.Name == sfc.flagName {
					switch element.OfType {
					case modeTreated:
						return _isTreated
					case modeControl:
						return _isControl
					default:
						return _isUnknown
					}
				}
			}
		}
	}
	return _isUnknown
}

/* This function will evaluate all expression nodes here */
func (sfc *staleFlagCleaner) evaluateExprNode(exprNode dst.Expr) Value {

	switch exprUnderConsideration := exprNode.(type) {
	case *dst.Ident:
		if val, ok := sfc.valueMap[exprUnderConsideration.Name]; ok {
			return val
		}
		switch exprUnderConsideration.Name {
		case strTrue:
			return isTrue
		case strFalse:
			return isFalse
		}

	// Handling expressions related to calling a function
	case *dst.CallExpr:
		// Get the type of API
		typeOfAPI := sfc.flagTypeAPI(exprUnderConsideration)
		// Hardcoding the output of some API types
		if typeOfAPI == _isTreated {
			if sfc.isTreated {
				return isTrue
			}
			return isFalse
		}
		if typeOfAPI == _isControl {
			if sfc.isTreated {
				return isFalse
			}
			return isTrue
		}
	//Binary expressions
	case *dst.BinaryExpr:
		valX := sfc.evaluateExprNode(exprUnderConsideration.X)
		valY := sfc.evaluateExprNode(exprUnderConsideration.Y)
		if valX != isUndefined || valY != isUndefined {
			switch exprUnderConsideration.Op {
			case token.LAND: //stands for &&
				if valX == isFalse || valY == isFalse {
					return isFalse
				}
				if valX == isTrue && valY == isTrue {
					return isTrue
				}
			case token.LOR: //stands for ||
				if valX == isTrue || valY == isTrue {
					return isTrue
				}
				if valX == isFalse && valY == isFalse {
					return isFalse
				}
			case token.EQL:
				if valX != isUndefined && valY != isUndefined {
					if valX == valY {
						return isTrue
					}
					return isFalse
				}

			case token.NEQ:
				if valX == valY {
					return isFalse
				}
				return isTrue
			}
		}
	//Unary expressions
	case *dst.UnaryExpr:
		switch exprUnderConsideration.Op {
		case token.NOT:
			valX := sfc.evaluateExprNode(exprUnderConsideration.X)
			switch valX {
			case isTrue:
				return isFalse
			case isFalse:
				return isTrue
			}
		case token.AND: // &expr
			return sfc.evaluateExprNode(exprUnderConsideration.X)
		default:
			//do nothing for now.
		}
	//Star expression (*expr)
	case *dst.StarExpr:
		return sfc.evaluateExprNode(exprUnderConsideration.X)
	//Paren Expr: exprs inside parenthesis
	case *dst.ParenExpr:
		return sfc.evaluateExprNode(exprUnderConsideration.X)

	//Selector Expr (expr.A, a.b, etc.)
	case *dst.SelectorExpr:
		switch ident := exprUnderConsideration.X.(type) {
		case *dst.Ident:
			if val, ok := sfc.valueMap[ident.Name+"."+exprUnderConsideration.Sel.Name]; ok {
				return val
			}
		default:
			//Do nothing for now
		}

	default:
		//Do nothing for now
	}

	return isUndefined
}

// This is the pre function of the dstutil.Apply
/*
From documentation:
If pre is not nil, it is called for each node before the node's children are
traversed (pre-order). If pre returns false, no children are traversed, and
not called for that node.
*/
func (sfc *staleFlagCleaner) pre(n *dstutil.Cursor) bool {
	return true
}

func (sfc *staleFlagCleaner) checkForBoolLiterals(value dst.Expr) bool {
	switch checkval := value.(type) {
	case *dst.Ident:
		if checkval.Name == strTrue || checkval.Name == strFalse {
			//do nothing
			return true
		}
	default:
		return false
	}
	return false
}

/*
Below function have this aim:
If Value is isUndefined or it is a bool literal, then we are not adding that to valueMap,
otherwise we will add it to valueMap.
This signifies whether variable has effect of flag or not.
*/
func (sfc *staleFlagCleaner) updateValueMapPost(name dst.Expr, value dst.Expr, doNotDelLiterals bool) {
	valOfExpr := sfc.evaluateExprNode(value)
	switch d := name.(type) {
	case *dst.Ident:
		if valOfExpr == isUndefined || doNotDelLiterals {
			if _, ok := sfc.valueMap[d.Name]; ok {
				delete(sfc.valueMap, d.Name)
			}
		} else {
			sfc.valueMap[d.Name] = valOfExpr
		}
	case *dst.SelectorExpr:
		switch ident := d.X.(type) {
		case *dst.Ident:
			if valOfExpr == isUndefined || doNotDelLiterals {
				if _, ok := sfc.valueMap[ident.Name+"."+d.Sel.Name]; ok {
					delete(sfc.valueMap, ident.Name+"."+d.Sel.Name)
				}
			} else {
				sfc.valueMap[ident.Name+"."+d.Sel.Name] = valOfExpr
			}
		default:
			//Do nothing for now
		}
	default:
		// other cases may come here in future
	}
}

/*
It will deal with *dst.ValueSpec. Values of variable stored in valueMap
will be deleted from node.
*/
func (sfc *staleFlagCleaner) DelValStmt(names *[]*dst.Ident, values *[]dst.Expr) bool {
	valOfExpr := isUndefined
	delCallExpr := false
	doNotDelLiterals := false
	for ind := 0; ind < len((*names)); ind++ {
		delCallExpr = false
		doNotDelLiterals = false
		if len((*values)) != 0 {
			valOfExpr = sfc.evaluateExprNode((*values)[ind])
			doNotDelLiterals = sfc.checkForBoolLiterals((*values)[ind])
			sfc.updateValueMapPost((*names)[ind], (*values)[ind], doNotDelLiterals)
		}

		if (valOfExpr != isUndefined && !doNotDelLiterals) || (*names)[ind].Name == sfc.flagName || delCallExpr {
			if len((*values)) != 0 {
				var trueIdent dst.Ident
				if valOfExpr == isTrue {
					trueIdent.Name = strTrue
				} else {
					trueIdent.Name = strFalse
				}
				(*values)[ind] = dst.Expr(&trueIdent)
			}
			if len((*names)) <= 1 {
				return true
			}
		}
		valOfExpr = isUndefined
	}
	return len((*names)) == 0
}

/*
It will deal with *dst.AssignStmt. Values of variable stored in valueMap
will be deleted from node.
*/
func (sfc *staleFlagCleaner) DelAssStmt(names *[]dst.Expr, values *[]dst.Expr) bool {
	valOfExpr := isUndefined
	delCallExpr := false
	doNotDelLiterals := false
	for ind := 0; ind < len((*names)); ind++ {
		//check if name is in valueName
		valOfExpr = sfc.evaluateExprNode((*values)[ind])
		delCallExpr = false

		doNotDelLiterals = sfc.checkForBoolLiterals((*values)[ind])

		// Need to see what to do with this thing
		sfc.updateValueMapPost((*names)[ind], (*values)[ind], doNotDelLiterals)

		if (valOfExpr != isUndefined && !doNotDelLiterals) || delCallExpr {
			var trueIdent dst.Ident
			if valOfExpr == isTrue {
				trueIdent.Name = strTrue
			} else {
				trueIdent.Name = strFalse
			}
			(*values)[ind] = dst.Expr(&trueIdent)
			if len((*names)) <= 1 {
				return true
			}
		}
	}
	return len((*names)) == 0
}

// This is the post function of the dstutil.Apply
/*
From documentation:
If post is not nil, and a prior call of pre didn't return false, post is
called for each node after its children are traversed (post-order). If
post returns false, traversal is terminated and Apply returns immediately.

Working:
This wiil be used to delete the nodes and storing the values of expressions.
*/
func (sfc *staleFlagCleaner) post(n *dstutil.Cursor) bool {
	switch d := n.Node().(type) {
	case *dst.ValueSpec:
		delVal := sfc.DelValStmt(&d.Names, &d.Values)
		if delVal {
			n.Delete()
		}
	//GenDecl
	case *dst.GenDecl:
		if len(d.Specs) == 0 {
			switch n.Parent().(type) {
			case *dst.DeclStmt:
				//wait Decl stmt
			default:
				n.Delete()
			}
		}
	//declstmt
	case *dst.DeclStmt:
		if gd := d.Decl.(*dst.GenDecl); gd.Tok == token.VAR {
			if len(gd.Specs) == 0 {
				n.Delete()
			}
		}

	//Assignment statements
	case *dst.AssignStmt:
		//then delete from slice
		sfc.DelAssStmt(&d.Lhs, &d.Rhs)

	//If cond statements
	case *dst.IfStmt:
		// Init of Ifstmt will get handled in assignStmt
		// Now evaluate the cond. Cond is already refactored due to pre-order traversal.
		cond := sfc.evaluateExprNode(d.Cond)
		if cond != isUndefined {
			if cond == isFalse {
				if d.Else != nil {
					for _, a := range d.Else.(*dst.BlockStmt).List {
						n.InsertBefore(a)
					}
				}
			} else {
				for _, a := range d.Body.List {
					n.InsertBefore(a)
				}
			}
			n.Delete()
		}
	//Switch statements
	case *dst.SwitchStmt:
		// Init of switchstmt will get handled in assignStmt
		if d.Tag != nil {
			valExpr := sfc.evaluateExprNode(d.Tag)
			if valExpr != isUndefined {
				notfoundCase := false
				for _, exprstmt := range d.Body.List {
					if exprstmt.(*dst.CaseClause).List != nil { //nil means default case
						valCaseClause := sfc.evaluateExprNode(exprstmt.(*dst.CaseClause).List[0])
						if valCaseClause == valExpr {
							//this has to go out as switch is going to delete
							for _, bodyEle := range exprstmt.(*dst.CaseClause).Body {
								n.InsertBefore(bodyEle)
							}
							notfoundCase = false
							break
						} else {
							notfoundCase = true
						}
					} else {
						//default case
						//here assuming defualt case is at last
						if notfoundCase {
							for _, bodyEle := range exprstmt.(*dst.CaseClause).Body {
								n.InsertBefore(bodyEle)
							}
						}
					}
				}
				n.Delete()
			}
		} else {
			//now switch is like elseif statements so do accordingly
			needDeletion := false
			isUndefinedAbove := false
			for ind := 0; ind < len(d.Body.List); ind++ {
				exprstmt := d.Body.List[ind]
				if exprstmt.(*dst.CaseClause).List != nil {
					valCaseClause := sfc.evaluateExprNode(exprstmt.(*dst.CaseClause).List[0])
					if valCaseClause == isTrue {
						//this has to go out as switch is going to delete
						//On the condition that there was no expression above whose value is isUndefined.
						if !isUndefinedAbove {
							for _, bodyEle := range exprstmt.(*dst.CaseClause).Body {
								n.InsertBefore(bodyEle)
							}
							needDeletion = true
							break
						} else {
							d.Body.List = d.Body.List[:ind+1]
							d.Body.List[ind].(*dst.CaseClause).List = nil
							break
						}
					} else if valCaseClause == isFalse {
						if len(d.Body.List) > 1 {
							copy(d.Body.List[ind:], d.Body.List[ind+1:])   // Shift d.Body.List[ind+1:] left one index.
							d.Body.List[len(d.Body.List)-1] = nil          // Erase last element (write zero value).
							d.Body.List = d.Body.List[:len(d.Body.List)-1] // Truncate slice.

							ind = ind - 1
						} else {
							n.Delete()
							break
						}
					} else {
						// so now value is bot
						isUndefinedAbove = true
					}
				}
			}
			if needDeletion {
				n.Delete()
			}
		}
	//handling for in general experssions
	case *dst.BinaryExpr:
		valX := sfc.evaluateExprNode(d.X)
		valY := sfc.evaluateExprNode(d.Y)
		if valX == isUndefined || valY == isUndefined {
			if valX == isUndefined && valY == isUndefined {
				//do nothing
			} else {
				//now one of them is not bot
				//check for valX
				if valX == isTrue && d.Op != token.LOR {
					n.Replace(d.Y)
				}
				if valX == isFalse && d.Op != token.LAND {
					n.Replace(d.Y)
				}
				if valY == isTrue && d.Op != token.LOR {
					n.Replace(d.X)
				}
				if valY == isFalse && d.Op != token.LAND {
					n.Replace(d.X)
				}
				if d.Op == token.EQL {
					newNode := n.Node()
					var trueIdent dst.Ident
					// either valX is not bot or valY. both cannot be bot at the same time as we checked out above
					if valX != isUndefined {
						if valX == isTrue {
							trueIdent.Name = strTrue
						} else {
							trueIdent.Name = strFalse
						}
						newNode.(*dst.BinaryExpr).X = dst.Expr(&trueIdent)
						n.Replace(newNode)
					} else {
						if valY == isTrue {
							trueIdent.Name = strTrue
						} else {
							trueIdent.Name = strFalse
						}
						newNode.(*dst.BinaryExpr).Y = dst.Expr(&trueIdent)
						n.Replace(newNode)
					}
				}
			}
		}
	default:

	}
	return true
}

// This is the Upper flow of the programme
/*
Input: Parsed File(dst.Node)
Output: New Parser File(dst.Node)

FLOW OF THE FUNCTION:

This will take parsed file which will contain all comments.
Walk the tree with apply. In Pre function get values of variable and fucntion
returns that are related to the flag. In Post funtion will delete the nodes
according to the conditions. Also, this will store elements for doing deep
clean pass.

Additional thought:

Thinking of adding deep clean functionality in here after completing the
First Phase of the code.
*/
func (sfc *staleFlagCleaner) run(root dst.Node) dst.Node {
	/*
		From documentation:
		Apply traverses a syntax tree recursively, starting with root, and
		calling pre and post for each node as described below. Apply returns
		the syntax tree, possibly modified.
	*/
	modifiedRoot := dstutil.Apply(root, sfc.pre, sfc.post)
	return modifiedRoot
}

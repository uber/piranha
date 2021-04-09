package com.uber.piranha;

import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableMultimap;
import com.google.common.collect.ImmutableSet;
import com.google.errorprone.util.ASTHelpers;
import com.sun.source.tree.AnnotationTree;
import com.sun.source.tree.AssignmentTree;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.LiteralTree;
import com.sun.source.tree.NewArrayTree;
import com.sun.source.tree.Tree;
import java.util.Locale;
import java.util.Optional;
import javax.annotation.Nullable;
import org.json.simple.JSONObject;

/** A class representing an annotation configuration record from properties.json */
final class PiranhaTestAnnotationSpecRecord {

  // Allowed fields for an annotation property in the config file.
  // Entered under the top-level "annotations" in properties.json.
  // By default, the name, flag, and treated fields are mandatory.
  // The group field is optional.
  private static final String ANNOTATION_NAME_KEY = "name";
  private static final String FLAG_IDENTIFIER_KEY = "flag";
  private static final String FLAG_TREATED_KEY = "treated";
  private static final String FLAG_GROUP_KEY = "group";
  private static final String[] REQUIRED_KEYS =
      new String[] {ANNOTATION_NAME_KEY, FLAG_IDENTIFIER_KEY, FLAG_TREATED_KEY};

  // We just need this to be an unique string which can never clash with a flag or group name, it's
  // used as a
  // special value to indicate a field of an annotation is not currently of a parseable type.
  private static final String UNPARSED_FIELD_VALUE =
      "$$$UNPARSED$$$9A1B0050-1BBD-420F-B6B3-AA0DB7D0AAE8";

  private final String annotationName;
  private final String flagIdentifierOrField;
  private final String treatedValueOrField;
  private final Optional<String> groupValueOrField;

  private PiranhaTestAnnotationSpecRecord(
      String annotationName,
      String flagIdentifierOrField,
      String treatedValueOrField,
      Optional<String> groupValueOrField) {
    this.annotationName = annotationName;
    this.flagIdentifierOrField = flagIdentifierOrField;
    this.treatedValueOrField = treatedValueOrField;
    this.groupValueOrField = groupValueOrField;
  }

  public String getAnnotationName() {
    return annotationName;
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

  public Optional<Resolved> resolveSpec(AnnotationTree at) {
    if (!at.getAnnotationType().toString().contains(annotationName)) {
      System.err.print(
          "Failed to resolve annotation "
              + at.getAnnotationType().toString()
              + " as named annotation "
              + annotationName);
      return Optional.empty();
    }
    ImmutableMultimap<String, AnnotationArgument> fields = parseAnnotationArguments(at);
    // Process flag id(s)
    ImmutableSet.Builder<AnnotationArgument> flagIdsBuilder = ImmutableSet.builder();
    if (this.flagIdentifierOrField.startsWith("$")) {
      // Annotation field reference
      if (!fields.containsKey(this.flagIdentifierOrField)) {
        // ToDo: Custom error class? (Actually, this one should likely be return Optional.empty())
        throw new Error(
            "Annotation named "
                + annotationName
                + " fails to meet spec in Piranha configuration file. Missing "
                + FLAG_IDENTIFIER_KEY
                + " field "
                + this.flagIdentifierOrField.substring(1)
                + "\nAnnotation instance:\n"
                + at.toString());
      }
      for (AnnotationArgument argument : fields.get(this.flagIdentifierOrField)) {
        // ToDo: Check for UNPARSED_FIELD_VALUE
        flagIdsBuilder.add(argument);
      }
    } else {
      // constant flag name
      flagIdsBuilder.add(new AnnotationArgument(this.flagIdentifierOrField, null));
    }
    // Process flag treatment value
    boolean treated;
    if (this.treatedValueOrField.startsWith("$")) {
      // ToDo: implement
      throw new Error("Field based treatment value in test annotations not yet supported");
    } else {
      // Only 'true' or 'false' are possible at this point, due to parsing logic
      Preconditions.checkArgument(
          this.treatedValueOrField.equals("true") || this.treatedValueOrField.equals("false"));
      treated = this.treatedValueOrField.equals("true");
    }
    // Process treatment group, if any
    ImmutableSet.Builder<AnnotationArgument> groupsBuilder = ImmutableSet.builder();
    if (groupValueOrField.isPresent()) {
      // ToDo: implement
      throw new Error("Treatment group information in test annotations not yet supported");
    }
    return Optional.of(new Resolved(flagIdsBuilder.build(), treated, groupsBuilder.build()));
  }

  private static String requireASStringValue(JSONObject annotationJSON, String key)
      throws PiranhaConfigurationException {
    if (!(annotationJSON.get(key) instanceof String)) {
      throw new PiranhaConfigurationException(
          "Unexpected value type (should be string) for key "
              + key
              + " in annotation specification record: "
              + annotationJSON.toString());
    }
    return (String) annotationJSON.get(key);
  }

  private static String requireASFieldRefOfBoolEquivalentValue(
      JSONObject annotationJSON, String key) throws PiranhaConfigurationException {
    Object val = annotationJSON.get(key);
    if (val instanceof String) {
      if (((String) val).startsWith("$")) {
        return (String) val; // correct
      }
      // accept true/True/TRUE and false/False/FALSE
      String lowerCaseVal = ((String) val).toLowerCase(Locale.ROOT);
      if (lowerCaseVal.equals("true") || lowerCaseVal.equals("false")) {
        return lowerCaseVal; // correct
      }
    } else if (val instanceof Boolean) {
      return (((Boolean) val) ? "true" : "false"); // correct
    }
    throw new PiranhaConfigurationException(
        "Unexpected value (should be a field reference starting with '$' or a boolean value) for key "
            + key
            + " in annotation specification record: "
            + annotationJSON.toString());
  }

  static PiranhaTestAnnotationSpecRecord fromJSONObject(JSONObject annotationJSON)
      throws PiranhaConfigurationException {
    for (String requiredKey : REQUIRED_KEYS) {
      if (!annotationJSON.containsKey(requiredKey)) {
        throw new PiranhaConfigurationException(
            "Missing required key "
                + requiredKey
                + " in annotation specification record: "
                + annotationJSON.toString());
      }
    }
    return new PiranhaTestAnnotationSpecRecord(
        requireASStringValue(annotationJSON, ANNOTATION_NAME_KEY),
        requireASStringValue(annotationJSON, FLAG_IDENTIFIER_KEY),
        requireASFieldRefOfBoolEquivalentValue(annotationJSON, FLAG_TREATED_KEY),
        annotationJSON.containsKey(FLAG_GROUP_KEY)
            ? Optional.of(requireASStringValue(annotationJSON, FLAG_GROUP_KEY))
            : Optional.empty());
  }

  static PiranhaTestAnnotationSpecRecord fromName(String name) {
    // For compatibility with old Piranha configs, default to an annotation of the form:
    // @AnnotationName(treated = [flag1, flag2])
    return new PiranhaTestAnnotationSpecRecord(name, "$treated", "true", Optional.empty());
  }

  public static final class AnnotationArgument {
    public final String value;
    @Nullable public final ExpressionTree sourceTree;

    public AnnotationArgument(String value, @Nullable ExpressionTree sourceTree) {
      this.value = value;
      this.sourceTree = sourceTree;
    }
  }

  public static final class Resolved {
    public final ImmutableSet<AnnotationArgument> flags;
    public final boolean treated;
    public final ImmutableSet<AnnotationArgument> groups;

    public Resolved(
        ImmutableSet<AnnotationArgument> flags,
        boolean treated,
        ImmutableSet<AnnotationArgument> groups) {
      this.flags = flags;
      this.treated = treated;
      this.groups = groups;
    }
  }
}

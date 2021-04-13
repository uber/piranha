package com.uber.piranha.testannotations;

import com.uber.piranha.config.PiranhaConfigurationException;
import java.util.Locale;
import java.util.Optional;
import org.json.simple.JSONObject;

/**
 * A class representing an annotation configuration record from properties.json
 *
 * <p>Visibility is package private as these should only be used by {@link TestAnnotationResolver},
 * with the core of Piranha using only the {@link ResolvedTestAnnotation} records instead.
 */
final class TestAnnotationSpecRecord {

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

  private final String annotationName;
  private final String flagIdentifierOrField;
  private final String treatedValueOrField;
  private final Optional<String> groupValueOrField;

  private TestAnnotationSpecRecord(
      String annotationName,
      String flagIdentifierOrField,
      String treatedValueOrField,
      Optional<String> groupValueOrField) {
    this.annotationName = annotationName;
    this.flagIdentifierOrField = flagIdentifierOrField;
    this.treatedValueOrField = treatedValueOrField;
    this.groupValueOrField = groupValueOrField;
  }

  String getAnnotationName() {
    return annotationName;
  }

  String getFlagIdentifierOrField() {
    return flagIdentifierOrField;
  }

  String getTreatedValueOrField() {
    return treatedValueOrField;
  }

  Optional<String> getGroupValueOrField() {
    return groupValueOrField;
  }

  /**
   * Abstract away checking that a value in the configuration file refers to a field.
   *
   * <p>Configuration convention is that a string like {@code "$foo"} refers to an annotation field
   * named {@code foo}.
   *
   * @param specValue the string representation of the annotation spec value.
   * @return {@code true} if this value represents a reference to a field in an annotation, {@code
   *     false} otherwise.
   */
  public static boolean isFieldReference(String specValue) {
    return specValue.startsWith("$");
  }

  @Override
  public String toString() {
    StringBuilder stringBuilder = new StringBuilder();
    stringBuilder.append("@");
    stringBuilder.append(annotationName);
    stringBuilder.append("(");
    if (isFieldReference(flagIdentifierOrField)) {
      stringBuilder.append(String.format("%s=$flags(s), ", flagIdentifierOrField.substring(1)));
    }
    if (isFieldReference(treatedValueOrField)) {
      stringBuilder.append(
          String.format("%s=$treatment_value, ", treatedValueOrField.substring(1)));
    }
    if (groupValueOrField.isPresent() && isFieldReference(groupValueOrField.get())) {
      stringBuilder.append(String.format("%s=$group(s), ", groupValueOrField.get().substring(1)));
    }
    stringBuilder.append("[...])");
    return stringBuilder.toString();
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

  static TestAnnotationSpecRecord fromJSONObject(JSONObject annotationJSON)
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
    return new TestAnnotationSpecRecord(
        requireASStringValue(annotationJSON, ANNOTATION_NAME_KEY),
        requireASStringValue(annotationJSON, FLAG_IDENTIFIER_KEY),
        requireASFieldRefOfBoolEquivalentValue(annotationJSON, FLAG_TREATED_KEY),
        annotationJSON.containsKey(FLAG_GROUP_KEY)
            ? Optional.of(requireASStringValue(annotationJSON, FLAG_GROUP_KEY))
            : Optional.empty());
  }

  static TestAnnotationSpecRecord fromName(String name) {
    // For compatibility with old Piranha configs, default to an annotation of the form:
    // @AnnotationName(treated = [flag1, flag2])
    return new TestAnnotationSpecRecord(name, "$treated", "true", Optional.empty());
  }
}

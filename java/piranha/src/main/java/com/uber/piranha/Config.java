package com.uber.piranha;

import com.google.common.collect.ImmutableCollection;
import com.google.common.collect.ImmutableMultimap;
import com.google.common.collect.ImmutableSet;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.Map;
import org.json.simple.JSONObject;
import org.json.simple.parser.JSONParser;
import org.json.simple.parser.ParseException;

/** Information provided in the properties.json config file. */
final class Config {
  // Default value when no properties.json file is provided or the linkURL field is missing
  private static final String DEFAULT_PIRANHA_URL = "https://github.com/uber/piranha";

  /* Names of top-level fields within properties.json */
  private static final String LINK_URL_KEY = "linkURL";
  private static final String ANNOTATIONS_KEY = "annotations";
  private static final String METHODS_KEY = "methodProperties";

  /**
   * configMethodsMap is a map where key is method name and value is a list where each item in the
   * list is a map that corresponds to each method property from properties.json. In most cases, the
   * list would have only one element. But if someone reuses the same method name with different
   * returnType/receiverType/argumentIndex, the list would have each method property map as one
   * element.
   */
  private final ImmutableMultimap<String, PiranhaMethodRecord> configMethodProperties;

  /**
   * testAnnotationSpecs is a map where key is the annotation name and the value is an object
   * encoding the specification of a testing annotations understood by Piranha, including how to
   * parse: a) the flag being tested, b) whether the test is requesting the flag in treatment or
   * control mode, c) (optionally) whether a particular treatment group is being specified
   */
  private final ImmutableMultimap<String, PiranhaTestAnnotationSpecRecord> testAnnotationSpecs;

  private final String linkURL;

  private Config(
      ImmutableMultimap<String, PiranhaMethodRecord> configMethodProperties,
      ImmutableMultimap<String, PiranhaTestAnnotationSpecRecord> testAnnotationSpecs,
      String linkURL) {
    this.configMethodProperties = configMethodProperties;
    this.testAnnotationSpecs = testAnnotationSpecs;
    this.linkURL = linkURL;
  }

  public ImmutableCollection<PiranhaMethodRecord> getMethodRecordsForName(String methodName) {
    return configMethodProperties.containsKey(methodName)
        ? configMethodProperties.get(methodName)
        : ImmutableSet.of();
  }

  public ImmutableMultimap<String, PiranhaTestAnnotationSpecRecord> getTestAnnotationSpecs() {
    return testAnnotationSpecs;
  }

  public String getLinkURL() {
    return linkURL;
  }

  static Config fromJSONFile(String configFile, boolean isArgumentIndexOptional)
      throws PiranhaConfigurationException {
    try {
      Path configFilePath = Paths.get(configFile);
      boolean configFileExists = configFilePath.toFile().exists();
      if (!configFileExists) {
        throw new IOException("Provided config file not found");
      }

      String linkURL = DEFAULT_PIRANHA_URL;
      ImmutableMultimap.Builder<String, PiranhaMethodRecord> methodsBuilder =
          ImmutableMultimap.builder();
      ImmutableMultimap.Builder<String, PiranhaTestAnnotationSpecRecord> annotationsBuilder =
          ImmutableMultimap.builder();

      JSONParser parser = new JSONParser();
      JSONObject propertiesJson =
          (JSONObject)
              parser.parse(Files.newBufferedReader(configFilePath, StandardCharsets.UTF_8));
      if (propertiesJson.containsKey(LINK_URL_KEY)) {
        linkURL = (String) propertiesJson.get(LINK_URL_KEY);
      }
      if (propertiesJson.get(ANNOTATIONS_KEY) != null) {
        for (Object annotationJSON : (List<Object>) propertiesJson.get(ANNOTATIONS_KEY)) {
          // ToDo: Two cases, String and Map<String, Object>
          PiranhaTestAnnotationSpecRecord annotSpec;
          if (annotationJSON instanceof String) {
            annotSpec = PiranhaTestAnnotationSpecRecord.fromName((String) annotationJSON);
          } else if (annotationJSON instanceof JSONObject) {
            annotSpec = PiranhaTestAnnotationSpecRecord.fromJSONObject((JSONObject) annotationJSON);
          } else {
            throw new PiranhaConfigurationException(
                "Unexpected annotation specification format inside "
                    + ANNOTATIONS_KEY
                    + " : "
                    + annotationJSON.toString());
          }
          annotationsBuilder.put(annotSpec.getAnnotationName(), annotSpec);
        }
      }
      if (propertiesJson.get(METHODS_KEY) != null) {
        for (Map<String, Object> methodProperty :
            (List<Map<String, Object>>) propertiesJson.get(METHODS_KEY)) {
          PiranhaMethodRecord methodRecord =
              PiranhaMethodRecord.parseFromJSONPropertyEntryMap(
                  methodProperty, isArgumentIndexOptional);
          methodsBuilder.put(methodRecord.getMethodName(), methodRecord);
        }
      } else {
        throw new PiranhaConfigurationException("methodProperties not found.");
      }
      return new Config(methodsBuilder.build(), annotationsBuilder.build(), linkURL);
    } catch (IOException fnfe) {
      throw new PiranhaConfigurationException(
          "Error reading config file " + Paths.get(configFile).toAbsolutePath() + " : " + fnfe);
    } catch (ParseException pe) {
      String extraWarning = "";
      if (configFile.endsWith(".properties")) {
        // Ends in space to make link clickable on terminal.
        extraWarning =
            "\nWARNING: With version 0.1.0, PiranhaJava has changed its configuration file format to json "
                + "(properties.json), but it looks you are passing the old piranha.properties format. Please "
                + "migrate your configuration to json. "
                + "See. https://github.com/uber/piranha/blob/master/java/README.md ";
      }
      throw new PiranhaConfigurationException(
          "Invalid or incorrectly formatted config file. " + pe + extraWarning);
    } catch (PiranhaConfigurationException pce) {
      // Already in the right format, re-throw
      throw pce;
    } catch (Exception e) {
      throw new PiranhaConfigurationException("Some other exception thrown while parsing config");
    }
  }

  static Config emptyConfig() {
    return new Config(ImmutableMultimap.of(), ImmutableMultimap.of(), DEFAULT_PIRANHA_URL);
  }
}

package com.uber.piranha.config;

import com.google.common.collect.ImmutableCollection;
import com.google.common.collect.ImmutableMultimap;
import com.google.common.collect.ImmutableSet;
import com.google.errorprone.VisitorState;
import com.sun.source.tree.MethodTree;
import com.uber.piranha.testannotations.ResolvedTestAnnotation;
import com.uber.piranha.testannotations.TestAnnotationResolver;
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
public final class Config {
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
   * testAnnotationResolver abstracts away Piranha's logic for configuring and resolving different
   * kinds of annotations used to specify which flags (and treatment conditions for flags) are being
   * tested by a particular unit test.
   */
  private final TestAnnotationResolver testAnnotationResolver;

  private final String linkURL;

  private Config(
      ImmutableMultimap<String, PiranhaMethodRecord> configMethodProperties,
      TestAnnotationResolver testAnnotationResolver,
      String linkURL) {
    this.configMethodProperties = configMethodProperties;
    this.testAnnotationResolver = testAnnotationResolver;
    this.linkURL = linkURL;
  }

  public ImmutableCollection<PiranhaMethodRecord> getMethodRecordsForName(String methodName) {
    return configMethodProperties.containsKey(methodName)
        ? configMethodProperties.get(methodName)
        : ImmutableSet.of();
  }

  public ImmutableSet<ResolvedTestAnnotation> resolveTestAnnotations(
      MethodTree tree, VisitorState state) {
    return testAnnotationResolver.resolveAllForMethod(tree, state);
  }

  public String getLinkURL() {
    return linkURL;
  }

  @SuppressWarnings("unchecked") // Not sure is there is a way to do checked parsing of JSONObject
  public static Config fromJSONFile(String configFile, boolean isArgumentIndexOptional)
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
      TestAnnotationResolver.Builder annotationResolverBuilder = TestAnnotationResolver.builder();

      JSONParser parser = new JSONParser();
      JSONObject propertiesJson =
          (JSONObject)
              parser.parse(Files.newBufferedReader(configFilePath, StandardCharsets.UTF_8));
      if (propertiesJson.containsKey(LINK_URL_KEY)) {
        linkURL = (String) propertiesJson.get(LINK_URL_KEY);
      }
      if (propertiesJson.get(ANNOTATIONS_KEY) != null) {
        for (Object annotationJSON : (List<Object>) propertiesJson.get(ANNOTATIONS_KEY)) {
          if (annotationJSON instanceof String) {
            annotationResolverBuilder.addFromName((String) annotationJSON);
          } else if (annotationJSON instanceof JSONObject) {
            annotationResolverBuilder.addFromJSONObject((JSONObject) annotationJSON);
          } else {
            throw new PiranhaConfigurationException(
                "Unexpected annotation specification format inside "
                    + ANNOTATIONS_KEY
                    + " : "
                    + annotationJSON.toString());
          }
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
      return new Config(methodsBuilder.build(), annotationResolverBuilder.build(), linkURL);
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

  public static Config emptyConfig() {
    return new Config(
        ImmutableMultimap.of(), TestAnnotationResolver.builder().build(), DEFAULT_PIRANHA_URL);
  }
}

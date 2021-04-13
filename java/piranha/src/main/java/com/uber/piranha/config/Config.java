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

  // Constructor is private, a Config object can be generated using the class' static methods,
  // in particular Config.fromJSONFile([properties.json])
  private Config(
      ImmutableMultimap<String, PiranhaMethodRecord> configMethodProperties,
      TestAnnotationResolver testAnnotationResolver,
      String linkURL) {
    this.configMethodProperties = configMethodProperties;
    this.testAnnotationResolver = testAnnotationResolver;
    this.linkURL = linkURL;
  }

  /**
   * Return all configuration method records matching a given method name.
   *
   * @param methodName the method name to search
   * @return A collection of {@link PiranhaMethodRecord} objects, representing each method
   *     definition in the piranha json configuration file matching {@code methodName}.
   */
  public ImmutableCollection<PiranhaMethodRecord> getMethodRecordsForName(String methodName) {
    return configMethodProperties.containsKey(methodName)
        ? configMethodProperties.get(methodName)
        : ImmutableSet.of();
  }

  /**
   * Resolve all experiment flag test annotations found on the given method.
   *
   * <p>Resolution involves searching for annotations matching the annotation specifications in the
   * Piranha json configuration file, and then using those specifications to abstract away the
   * particular field names, types, and general shape of the actual annotation into a canonical
   * {@link ResolvedTestAnnotation} record.
   *
   * @param tree the AST tree representing the annotated method
   * @param state the visitor state (for AST to source code resolution)
   * @return All annotations on tree which represent flag test annotations, properly resolved to
   *     {@link ResolvedTestAnnotation} records.
   */
  public ImmutableSet<ResolvedTestAnnotation> resolveTestAnnotations(
      MethodTree tree, VisitorState state) {
    return testAnnotationResolver.resolveAllForMethod(tree, state);
  }

  /**
   * Get the link url from the Piranha json configuration file.
   *
   * @return The string value of {@code linkURL} in the json configuration file.
   */
  public String getLinkURL() {
    return linkURL;
  }

  /**
   * Parse a Piranha json configuration file into a Config object.
   *
   * <p>This is the canonical way of producing a Config object for Piranha to use.
   *
   * @param configFile The full path to Piranha's properties.json or piranha.json
   * @param isArgumentIndexOptional Whether the argument index for method records is to be
   *     considered optional (defaulting to 0) when parsing the json configuration. Set to {@code
   *     false} for stricter configuration parsing.
   * @return A {@link Config} instance matching the information in the given config file.
   * @throws PiranhaConfigurationException If the file passed doesn't contain a valid Piranha json
   *     configuration.
   */
  @SuppressWarnings("unchecked") // Not sure is there is a way to do checked parsing of JSONObject
  public static Config fromJSONFile(String configFile, boolean isArgumentIndexOptional)
      throws PiranhaConfigurationException {
    try {
      Path configFilePath = Paths.get(configFile);
      boolean configFileExists = configFilePath.toFile().exists();
      if (!configFileExists) {
        // We will catch this with other IOExceptions and wrap it around as
        // PiranhaConfigurationException (See catch block)
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
            annotationResolverBuilder.addSpecFromName((String) annotationJSON);
          } else if (annotationJSON instanceof JSONObject) {
            annotationResolverBuilder.addSpecFromJSONObject((JSONObject) annotationJSON);
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
        throw new PiranhaConfigurationException("methodProperties not found, required.");
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
      // Already in the right format, re-throw, to avoid falling on catch-all below.
      throw pce;
    } catch (Exception e) {
      throw new PiranhaConfigurationException("Some other exception thrown while parsing config");
    }
  }

  /**
   * Returns an empty {@link Config} object, used to instantiate Piranha when checking is
   * <b>disabled</b>.
   *
   * <p>The empty {@link Config} lists no known methods, no known test annotations, and gives
   * Piranha's default reference URL.
   *
   * @return An empty {@link Config} object.
   */
  public static Config emptyConfig() {
    return new Config(
        ImmutableMultimap.of(), TestAnnotationResolver.builder().build(), DEFAULT_PIRANHA_URL);
  }
}

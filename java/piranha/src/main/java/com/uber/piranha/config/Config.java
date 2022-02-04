/**
 * Copyright (c) 2021 Uber Technologies, Inc.
 *
 * <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 * except in compliance with the License. You may obtain a copy of the License at
 *
 * <p>http://www.apache.org/licenses/LICENSE-2.0
 *
 * <p>Unless required by applicable law or agreed to in writing, software distributed under the
 * License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 * express or implied. See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.uber.piranha.config;

import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableCollection;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableMultimap;
import com.google.common.collect.ImmutableSet;
import com.google.errorprone.VisitorState;
import com.google.errorprone.matchers.Matchers;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.MemberSelectTree;
import com.sun.source.tree.MethodInvocationTree;
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
import java.util.Optional;
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
  private static final String UNNECESSARY_TEST_METHOD_KEY = "unnecessaryTestMethodProperties";
  private static final String CLEANUP_OPTS_KEY = "cleanupOptions";
  private static final String ENUMS_KEY = "enumProperties";

  /* Named clean up options within the cleanupOptions property, all are optional */
  private static final String OPT_TESTS_CLEAN_BY_SETTERS_ENABLED =
      "tests.clean_by_setters_heuristic.enabled";
  private static final String OPT_TESTS_CLEAN_BY_SETTERS_LIMIT =
      "tests.clean_by_setters_heuristic.lines_limit";
  private static final String OPT_TESTS_CLEAN_BY_SETTERS_IGNORE_OTHERS =
      "tests.clean_by_setters_heuristic.ignore_other_flag_sets";
  private static final String ALLOW_METHOD_CHAIN = "allow_method_chain";

  private static final String FLAG_METHOD_NAME = "flag_method_name";

  private static final ImmutableSet<String> ALL_OPTS =
      ImmutableSet.of(
          OPT_TESTS_CLEAN_BY_SETTERS_ENABLED,
          OPT_TESTS_CLEAN_BY_SETTERS_LIMIT,
          OPT_TESTS_CLEAN_BY_SETTERS_IGNORE_OTHERS,
          ALLOW_METHOD_CHAIN,
          FLAG_METHOD_NAME);

  /* Defaults for named clean up options */
  private static final boolean DEFAULT_TESTS_CLEAN_BY_SETTERS_ENABLED = false;
  private static final long DEFAULT_TESTS_CLEAN_BY_SETTERS_LIMIT = 100;
  private static final boolean DEFAULT_ALLOW_METHOD_CHAIN = false;
  private static final boolean DEFAULT_TESTS_CLEAN_BY_SETTERS_IGNORE_OTHERS = false;

  /**
   * configMethodProperties is a map where key is method name and value is a list where each item in
   * the list is a map that corresponds to each method property from properties.json. In most cases,
   * the list would have only one element. But if someone reuses the same method name with different
   * returnType/receiverType/argumentIndex, the list would have each method property map as one
   * element.
   */
  private final ImmutableMultimap<String, PiranhaMethodRecord> configMethodProperties;

  /**
   * unnecessaryTestMethodProperties is a map where key is method name and value is a list where
   * each item in the list is a map that corresponds to each test method property from
   * properties.json. In most cases, the list would have only one element. But if someone reuses the
   * same method name with different returnType/receiverType/argumentIndex, the list would have each
   * method property map as one element. Often after the refactoring performed by Piranha, certain
   * method invocations in test cases become unnecessary, need to be deleted.
   */
  private final ImmutableMultimap<String, MethodRecord> unnecessaryTestMethodProperties;
  /**
   * configEnumProperties is a map where key is enum name and value is a list where each item in the
   * list is a map that corresponds to each enum property from properties.json. In most cases, the
   * list would have only one element. But if someone reuses the same enum name with different
   * argumentIndex, the list would have each enum property map as one element.
   */
  private final ImmutableMultimap<String, PiranhaEnumRecord> configEnumProperties;

  /**
   * testAnnotationResolver abstracts away Piranha's logic for configuring and resolving different
   * kinds of annotations used to specify which flags (and treatment conditions for flags) are being
   * tested by a particular unit test.
   */
  private final TestAnnotationResolver testAnnotationResolver;

  /**
   * cleanupOptions stores the misc cleanup configuration options stored in the cleanupOptions
   * section of properties.json. These are generally a set of heuristics for Piranha to use in
   * particular scenarios.
   */
  private final ImmutableMap<String, Object> cleanupOptions;

  private final String linkURL;

  // Constructor is private, a Config object can be generated using the class' static methods,
  // in particular Config.fromJSONFile([properties.json])
  private Config(
      ImmutableMultimap<String, PiranhaMethodRecord> configMethodProperties,
      ImmutableMultimap<String, MethodRecord> unnecessaryTestMethodProperties,
      ImmutableMultimap<String, PiranhaEnumRecord> configEnumProperties,
      TestAnnotationResolver testAnnotationResolver,
      ImmutableMap<String, Object> cleanupOptions,
      String linkURL) {
    this.configMethodProperties = configMethodProperties;
    this.unnecessaryTestMethodProperties = unnecessaryTestMethodProperties;
    this.configEnumProperties = configEnumProperties;
    this.testAnnotationResolver = testAnnotationResolver;
    this.cleanupOptions = cleanupOptions;
    this.linkURL = linkURL;
  }

  /**
   * Return all configuration method records matching the name of the given method invocation tree.
   *
   * <p>If the cleanup option "allow_method_chain" is set to true, returns all configurations method
   * record matching the name of the given method invocation and its receiver method invocation. For
   * instance, the invocation `exp.stale_flag().getValue()` will match the method record with name
   * as "stale_flag.getValue". Note: This method only supports matching a method chain of length 2
   * Moreover, it only matches chained invocations where the nested invocation is an instance method
   * invocation (with a receiver). For instance, abc.stale_flag().getValue() and not
   * stale_flag().getValue().
   *
   * @param mit Method invocation AST
   * @param state visitor state
   * @return A collection of {@link PiranhaMethodRecord} objects, representing each method
   *     definition in the piranha json configuration file matching {@code methodName}.
   */
  public ImmutableCollection<PiranhaMethodRecord> getMethodRecordsForName(
      MethodInvocationTree mit, VisitorState state) {
    String methodName = getMethodName(mit);
    if (configMethodProperties.containsKey(methodName)) {
      return configMethodProperties.get(methodName);
    }

    // Check if mit matches a method record for a method chain
    ExpressionTree methodSelect = mit.getMethodSelect();
    if (allowMethodChain() && methodSelect instanceof MemberSelectTree) {
      ExpressionTree mstExpr = ((MemberSelectTree) methodSelect).getExpression();
      if (mstExpr instanceof MethodInvocationTree) {
        MethodInvocationTree chainedMIT = (MethodInvocationTree) mstExpr;
        String chainedMethodName = getMethodName(chainedMIT) + "." + methodName;
        // This ensures that we only match instance method invocations like
        // abc.stale_flag().getValue() and not stale_flag().getValue()
        if (chainedMIT.getMethodSelect() instanceof MemberSelectTree
            && Matchers.instanceMethod().anyClass().matches(chainedMIT, state)
            && configMethodProperties.containsKey(chainedMethodName))
          return configMethodProperties.get(chainedMethodName);
      }
    }

    return ImmutableSet.of();
  }

  /**
   * Returns name of the method invocation
   *
   * @param mit Method invocation tree
   * @return a string representing the name of the method invocation
   */
  public String getMethodName(MethodInvocationTree mit) {
    if (mit.getMethodSelect() instanceof MemberSelectTree) {
      MemberSelectTree mst = (MemberSelectTree) mit.getMethodSelect();
      return mst.getIdentifier().toString();
    }
    // scenario when there is no receiver, like foo("bar")
    return mit.getMethodSelect().toString();
  }

  /**
   * Return all the configured unnecessary test methods (corresponding to the
   * unnecessaryTestMethodProperties field)
   *
   * @return A collection of {@link MethodRecord} objects, representing each method definition in
   *     the piranha json configuration file (unnecessaryTestMethodProperties field) matching {@code
   *     methodName}.
   */
  public ImmutableCollection<MethodRecord> getUnnecessaryTestMethodRecords() {
    return unnecessaryTestMethodProperties.values();
  }

  /**
   * Returns whether any configuration enum records exist. Useful for skipping logic if enum
   * properties are not configured.
   *
   * @return Whether any "enumProperties" records were configured
   */
  public boolean hasEnumRecords() {
    return !configEnumProperties.isEmpty();
  }

  /**
   * Return all configuration enum records matching a given enum name.
   *
   * @param enumName the enum name to search
   * @return A collection of {@link PiranhaEnumRecord} objects, representing each enum definition in
   *     the piranha json configuration file matching {@code enumName}.
   */
  public ImmutableCollection<PiranhaEnumRecord> getEnumRecordsForName(String enumName) {
    return configEnumProperties.containsKey(enumName)
        ? configEnumProperties.get(enumName)
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

  // Start of OPT_* retrieval methods

  /**
   * Whether or not Piranha should try to clean up unit test methods based on the API calls inside
   * the method.
   *
   * <p>Generally, this will be based on instances of set_treated and set_control API methods
   *
   * @return a boolean representing whether this type of clean up is enabled or not.
   */
  public boolean shouldCleanTestMethodsByContent() {
    return (boolean)
        cleanupOptions.getOrDefault(
            OPT_TESTS_CLEAN_BY_SETTERS_ENABLED, DEFAULT_TESTS_CLEAN_BY_SETTERS_ENABLED);
  }

  public long testMethodCleanupSizeLimit() {
    return (long)
        cleanupOptions.getOrDefault(
            OPT_TESTS_CLEAN_BY_SETTERS_LIMIT, DEFAULT_TESTS_CLEAN_BY_SETTERS_LIMIT);
  }

  public boolean shouldIgnoreOtherSettersWhenCleaningTests() {
    return (boolean)
        cleanupOptions.getOrDefault(
            OPT_TESTS_CLEAN_BY_SETTERS_IGNORE_OTHERS, DEFAULT_TESTS_CLEAN_BY_SETTERS_IGNORE_OTHERS);
  }

  public boolean allowMethodChain() {
    return (boolean) cleanupOptions.getOrDefault(ALLOW_METHOD_CHAIN, DEFAULT_ALLOW_METHOD_CHAIN);
  }

  public Optional<String> getFlagMethodName() {
    return Optional.ofNullable((String) cleanupOptions.get(FLAG_METHOD_NAME));
  }

  // End of OPT_* retrieval methods

  private static String validateConfigOptsKey(Object k) {
    if (!(k instanceof String)) {
      throw new PiranhaConfigurationException(
          String.format(
              "Invalid key inside cleanupOptions: %s (clean up option keys must be strings)",
              k.toString()));
    }
    if (!ALL_OPTS.contains(k)) {
      throw new PiranhaConfigurationException(
          String.format(
              "Unrecognized key inside cleanupOptions: %s (not a valid Piranha cleanup option)",
              k.toString()));
    }
    return (String) k;
  }

  private static void requireType(String valK, Object v, Class<? extends Object> klass) {
    if (!klass.isInstance(v)) {
      throw new PiranhaConfigurationException(
          String.format(
              "Invalid value %s for key %s inside cleanupOptions (expected type %s)",
              v.toString(), valK, klass.toString()));
    }
  }

  private static void validateConfigOptsValue(String valK, Object v) {
    if (OPT_TESTS_CLEAN_BY_SETTERS_ENABLED.equals(valK)
        || OPT_TESTS_CLEAN_BY_SETTERS_IGNORE_OTHERS.equals(valK)
        || ALLOW_METHOD_CHAIN.equals(valK)) {
      requireType(valK, v, Boolean.class);
    } else if (OPT_TESTS_CLEAN_BY_SETTERS_LIMIT.equals(valK)) {
      requireType(valK, v, Long.class);
    } else if (FLAG_METHOD_NAME.equals(valK)) {
      requireType(valK, v, String.class);
    } else {
      Preconditions.checkArgument(false, "Default case should be unreachable.");
    }
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
      ImmutableMultimap.Builder<String, MethodRecord> unnecessaryTestMethodsBuilder =
          ImmutableMultimap.builder();
      ImmutableMultimap.Builder<String, PiranhaMethodRecord> flagMethodsBuilder =
          ImmutableMultimap.builder();
      ImmutableMultimap.Builder<String, PiranhaEnumRecord> enumsBuilder =
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
          flagMethodsBuilder.put(methodRecord.getMethodName(), methodRecord);
        }
        if (propertiesJson.get(UNNECESSARY_TEST_METHOD_KEY) != null) {
          for (Map<String, Object> methodProperty :
              (List<Map<String, Object>>) propertiesJson.get(UNNECESSARY_TEST_METHOD_KEY)) {
            MethodRecord methodRecord =
                MethodRecord.parseFromJSONPropertyEntryMap(methodProperty, isArgumentIndexOptional);
            unnecessaryTestMethodsBuilder.put(methodRecord.getMethodName(), methodRecord);
          }
        }
      } else {
        throw new PiranhaConfigurationException("methodProperties not found, required.");
      }
      if (propertiesJson.get(ENUMS_KEY) != null) {
        for (Map<String, Object> enumProperty :
            (List<Map<String, Object>>) propertiesJson.get(ENUMS_KEY)) {
          PiranhaEnumRecord enumRecord =
              PiranhaEnumRecord.parseFromJSONPropertyEntryMap(
                  enumProperty, isArgumentIndexOptional);
          enumsBuilder.put(enumRecord.getEnumName(), enumRecord);
        }
      }
      ImmutableMap.Builder<String, Object> cleanupOptionsBuilder = ImmutableMap.builder();
      final Object configOptsObj = propertiesJson.get(CLEANUP_OPTS_KEY);
      if (configOptsObj != null && configOptsObj instanceof JSONObject) {
        final JSONObject configOpts = (JSONObject) configOptsObj;
        configOpts.forEach(
            (k, v) -> {
              String valK = validateConfigOptsKey(k);
              validateConfigOptsValue(valK, v);
              cleanupOptionsBuilder.put(valK, v);
            });
      }
      return new Config(
          flagMethodsBuilder.build(),
          unnecessaryTestMethodsBuilder.build(),
          enumsBuilder.build(),
          annotationResolverBuilder.build(),
          cleanupOptionsBuilder.build(),
          linkURL);
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
        ImmutableMultimap.of(),
        ImmutableMultimap.of(),
        ImmutableMultimap.of(),
        TestAnnotationResolver.builder().build(),
        ImmutableMap.of(),
        DEFAULT_PIRANHA_URL);
  }
}

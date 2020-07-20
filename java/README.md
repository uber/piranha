# PiranhaJava

## Installation

### Overview

Piranha requires that you build your code with [Error Prone](http://errorprone.info), version 2.4.0 or higher.  See the [Error Prone documentation](http://errorprone.info/docs/installation) for instructions on getting started with Error Prone and integration with your build system.  

While not required, we strongly recommend that you use Piranha in combination with an automated code formatter, such as [Google Java Format](https://github.com/google/google-java-format) running as a pre-commit hook. This is because Piranha will transform code without any particular way to configure code style guidelines into the tool directly. While we strive to produce clean and readable refactorings by default, no commitment exists to any particular whitespace, line length, or styling behavior, nor even consistency between Piranha minor versions.

### Gradle

To integrate Piranha into your Java project you'll need a version of the following additions to your `build.gradle` file:

```groovy
plugins {
  id "com.github.sherter.google-java-format" version "0.7.1"
  id "net.ltgt.errorprone" version "0.6" apply false
  id "java"
}

sourceCompatibility = "1.8"
targetCompatibility = "1.8"

dependencies {
  annotationProcessor "com.uber.piranha:piranha:0.1.0"
  errorprone "com.google.errorprone:error_prone_core:2.4.0"
  errorproneJavac "com.google.errorprone:javac:9+181-r4173-1"
}

import net.ltgt.gradle.errorprone.CheckSeverity

tasks.withType(JavaCompile) {
  options.errorprone {
      check("Piranha", CheckSeverity.WARN)
  }
  options.errorprone.errorproneArgs << "-XepPatchChecks:Piranha"
  options.errorprone.errorproneArgs << "-XepPatchLocation:IN_PLACE"
  // The lines below should be replaced by code that loads the specific flag to patch
  // and final treatment condition.
  options.errorprone.errorproneArgs << "-XepOpt:Piranha:FlagName=SAMPLE_STALE_FLAG"
  options.errorprone.errorproneArgs << "-XepOpt:Piranha:IsTreated=true"
  options.errorprone.errorproneArgs << "-XepOpt:Piranha:Config=config/properties.json"
}
```

The `plugins` section pulls in the [Gradle Error Prone plugin](https://github.com/tbroyer/gradle-errorprone-plugin) for Error Prone integration. In `dependencies`, the `annotationProcessor` line loads Piranha, the `errorprone` line ensures that a compatible version of Error Prone is used, and the `errorproneJavac` line is needed for JDK 8 compatibility.  

In the `tasks.withType(JavaCompile)` section, we pass some configuration options to Piranha.  First `check("Piranha", CheckSeverity.WARN)` sets Piranha issues to the warning level. Then, `option.errorprone.errorproneArgs` is used to add a set of arguments to Piranha. `XepPatchChecks:Piranha` and `-XepPatchLocation:IN_PLACE` arguments are used together to enable in-place refactoring of the code. `-XepOpt:Piranha:FlagName` is used to specify a stale flag name that is used in the code, `-XepOpt:Piranha:IsTreated` is used to specify whether the treatment (`true`) branch or the control (`false`) branch needs to be taken during refactoring. Then `-XepOpt:Piranha:Config` is used to provide the properties file which specifies the APIs and annotations that are considered for refactoring. 

The properties file has the following template: 

```json
{
  "methodProperties":
    [
      {
        "methodName": "isToggleEnabled",
        "flagType": "treated",
        "returnType": "boolean",
        "receiverType": "com.uber.piranha.XPTest",
        "argumentIndex": 0
      },
      ...
    ],
  "linkURL": "<provide_your_url>",
  "annotations": ["ToggleTesting"]
}
```

The required top-level field is `methodProperties`.
Within that, there is an array of JSON objects, having the required fields `methodName`, `flagType` and `argumentIndex`.
The optional fields are `returnType`, `receiverType`.

The `flagType` with `treated` are the APIs which correspond to the treatment behavior of the flag, `control` correspond to the control behavior of the flag. In the above example, the API `flagEnabled` corresponds to treatment behavior. Hence, when the `IsTreated` Piranha argument is set to `true`, `flagEnabled(SAMPLE_STALE_FLAG)` will be evaluated to `true`. Similarly, `flagDisabled(SAMPLE_STALE_FLAG)` which corresponds to the control behavior will evaluate to `false`. 
The `flagType` with `empty` specifies the APIs which need to be discarded from the code. For example, if `enableFlag` is listed as a method in properties.json, with `flagType=empty`, a statement `enableFlag(SAMPLE_STALE_FLAG);` will be deleted from the code. 

The `argumentIndex` field specifies where to look for the flag name (given by `-XepOpt:Piranha:FlagName`) in the method's arguments. We follow 0 based indexing.
If your toggle methods take no arguments, or if you want to delete all occurrences of a given `methodName` irrespective of their arguments, you can set the `Piranha:ArgumentIndexOptional` to `true` to make specifying `argumentIndex` optional.

For `returnType` and `receiverType`, types should be written as `boolean` or `void` for primitive types, and fully qualified for custom defined types. eg: `com.uber.piranha.XPTest` or `java.lang.String` (not case-sensitive)

The `annotations` specify the annotations used (e.g., in unit testing) to determine treatment or control behavior. For example:

```java
@FlagTesting(treated = TestExperimentName.SAMPLE_STALE_FLAG)
public void some_unit_test() { ... }
```

will be refactored to 

```java
public void some_unit_test() { ... }
```

when `IsTreated` is `true`, and will be deleted completely when `IsTreated` is `false`. 

Finally, the setting `linkURL` in the properties file is to provide a URL describing the Piranha tooling and any custom configurations associated with the codebase. 


## Example refactoring

Consider a simple example

```java
public class MyClass {
  private XPTest expt;
  ...
  public void foo() {
    if(expt.flagEnabled(TestExperimentName.SAMPLE_STALE_FLAG)) {
        System.out.println("Hello World");
    }
  }

  public void bar() {
    if(expt.flagDisabled(TestExperimentName.SAMPLE_STALE_FLAG)) {
        System.out.println("Hi World");
    }
  }
}
```

and the following arguments to Piranha

```groovy
options.errorprone.errorproneArgs << "-XepOpt:Piranha:FlagName=SAMPLE_STALE_FLAG"
options.errorprone.errorproneArgs << "-XepOpt:Piranha:IsTreated=true"
options.errorprone.errorproneArgs << "-XepOpt:Piranha:Config=config/properties.json
```
where `properties.json` contains the following, 

```json
{
  "methodProperties":
    [
      {
        "methodName": "flagEnabled",
        "flagType": "treated",
        "argumentIndex": 0
      },
      {
        "methodName": "flagDisabled",
        "flagType": "control",
        "argumentIndex": 0
      },
      {
        "methodName": "enableFlag",
        "flagType": "empty",
        "argumentIndex": 0
      },
      {
        "methodName": "disableFlag",
        "flagType": "empty",
        "argumentIndex": 0
      }
    ],
  "linkURL": "<provide_your_url>",
  "annotations": ["FlagTesting"]
}
```

the refactored output will be 

```java
public class MyClass {
  private XPTest expt;
  ...
  public void foo() {
     System.out.println("Hello World");
  }
  
  public void bar() {
  }
}
```

When `IsTreated` is `false`, then the refactored output will be 

```java
public class MyClass {
  private XPTest expt;
  ...
  public void foo() {
  }
  
  public void bar() {
    System.out.println("Hi World");
  }
}
```

This example is present in the [sample](https://github.com/uber/piranha/tree/master/java/sample/) directory. 

IMPORTANT: Please note that the gradle build script included in that directory assumes that the sample will be built as part of the full Piranha Java build project (i.e. it depends on other gradle files within that project and assumes some setup done by them). If you wish to build the `sample` project as a standalone, you might need to recreate the `build.gradle` file included there using the instructions elsewhere in this readme.

### Maven Instructions

* For the example Piranha configuration discussed above, follow steps given for the ErrorProne [example](https://github.com/google/error-prone/blob/master/examples/maven/pom.xml) to setup the `pom.xml` to run with ErrorProne.

* Update the `pom.xml` with Piranha related configuration. It will be as follows: 
```xml
  <build>
    <plugins>
      <plugin>
        <groupId>org.apache.maven.plugins</groupId>
        <artifactId>maven-compiler-plugin</artifactId>
        <version>3.8.0</version>
        <configuration>
          <source>8</source>
          <target>8</target>
           <showWarnings>true</showWarnings>
          <compilerArgs>
            <arg>-XDcompilePolicy=simple</arg>
            <arg>-Xplugin:ErrorProne -Xep:Piranha:WARN -XepPatchChecks:Piranha -XepPatchLocation:IN_PLACE -XepOpt:Piranha:FlagName=SAMPLE_STALE_FLAG -XepOpt:Piranha:IsTreated=true -XepOpt:Piranha:Config=config/properties.json</arg>
          </compilerArgs>
          <annotationProcessorPaths>
            <path>
              <groupId>com.uber.piranha</groupId>
              <artifactId>piranha</artifactId>
              <version>0.1.0</version>
            </path>
            <path>
              <groupId>com.google.errorprone</groupId>
              <artifactId>error_prone_core</artifactId>
              <version>2.4.0</version>
            </path>
          </annotationProcessorPaths>
        </configuration>
      </plugin>
    </plugins>
  </build>
```

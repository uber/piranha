package com.uber.piranha.config;

/**
 * An exception thrown when Piranha's configuration is invalid. This is a runtime exception and will
 * crash piranha, as the tool can't proceed with incorrect configuration.
 */
public final class PiranhaConfigurationException extends RuntimeException {

  public PiranhaConfigurationException(String message) {
    super(message);
  }
}

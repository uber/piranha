package com.uber.piranha;

/**
 * An exception thrown when Piranha's configuration is invalid. This is a runtime exception and will
 * crash piranha, as the tool can't proceed with incorrect configuration.
 */
final class PiranhaConfigurationException extends RuntimeException {

  public PiranhaConfigurationException(String message) {
    super(message);
  }
}

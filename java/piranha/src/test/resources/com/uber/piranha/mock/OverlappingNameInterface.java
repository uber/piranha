package com.uber.piranha;

interface OverlappingNameInterface {
  public boolean staleFlag();

  static OverlappingNameInterface create(Parameter cp) {
    return null;
  }
}

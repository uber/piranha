package com.uber.piranha

interface SceneStateParameters {
    
    fun create(cachedParameters: CachedParameters): SceneStateParameters {
      return  SceneStateParametersProvider.create(cachedParameters)
    }
  }

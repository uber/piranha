package com.uber.piranha

import com.uber.something.piranha.ParameterUtils


interface SceneStateParameters {

    fun create(cachedParameters: CachedParameters): SceneStateParameters {
      return ParameterUtils.create("Test", cachedParameters)
    }
  }
}

from tensorflow.keras import layers
sp = layers.builder.config("config3", "1").config("config1", "5").config("config2", "5").getOrCreate()

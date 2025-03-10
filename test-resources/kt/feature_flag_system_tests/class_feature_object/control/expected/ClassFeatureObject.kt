class Sample {

    override fun isStaleFeatureEnabled() {
        return true
    }

    fun someMethod() {
        if (isStaleFeatureEnabled()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }
    }

    companion object {
    }
}

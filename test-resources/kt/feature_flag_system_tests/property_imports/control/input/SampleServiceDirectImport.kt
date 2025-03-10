class FeatureUser {

    fun main() {
        if (isEnabled(SampleClassWithProperty.FEATURE)) {
            print("Hi world!")
            return
        }
        val transactionId = event.internalEvent.accountingEntry.id.value.value
        print("Hello world!")
    }
}

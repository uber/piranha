sealed class FeatureToggle(val name: String)

sealed class FeatureFlag(val name: String) {
}

object ActivateFinCrimeChecks : FeatureToggle("should_activate_fin_crime_checks")
object ActivateSecondaryAccounts : FeatureToggle("should_activate_secondary_accounts")
object ActivateCarboniumConsumer : FeatureToggle("should_activate_carbonium_consumer")
object ActivateFailStuckPaymentsJob : FeatureToggle("should_activate_fail_stuck_payments_job")

object ObjectFeatures {
    const val AnotherFlag = "ANOTHER_FLAG"
}

sealed class ComplexFeatures(val flag: FeatureToggle, val something: String) {
    data object STALE_FLAG_COMPLEX : ComplexFeatures(ActivateFinCrimeChecks, "test")
}

sealed class ComplexFeatures2(val flag: ObjectFeatures, val something: String) {
    data object STALE_FLAG_COMPLEX : ComplexFeatures(AnotherFlag, "test")
}

fun main() {
    println("Hello world!")
    println("Hi world!")
    println("Hello world!")
}

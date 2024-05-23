sealed class FeatureToggle(val name: String)

object ActivateFinCrimeChecks : FeatureToggle("should_activate_fin_crime_checks")
object ActivateSecondaryAccounts : FeatureToggle("should_activate_secondary_accounts")
object ActivateCarboniumConsumer : FeatureToggle("should_activate_carbonium_consumer")
object ActivateFailStuckPaymentsJob : FeatureToggle("should_activate_fail_stuck_payments_job")

fun main() {
    println("Hello world!")
    println("Hi world!")
    println("Hello world!")
}

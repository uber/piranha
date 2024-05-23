sealed class FeatureToggle(val name: String)

object ActivateFinCrimeChecks : FeatureToggle("should_activate_fin_crime_checks")
object ActivateSecondaryAccounts : FeatureToggle("should_activate_secondary_accounts")
object ActivateCarboniumConsumer : FeatureToggle("should_activate_carbonium_consumer")
object ActivateFailStuckPaymentsJob : FeatureToggle("should_activate_fail_stuck_payments_job")
object ActivateStaleFlagJob : FeatureToggle("STALE_FLAG")


fun isFeatureEnabled_SealedClass() =
    featureService.isEnabled(ActivateStaleFlagJob)

fun isNotFeatureEnabled_SealedClass() =
    featureService.isEnabled(ActivateStaleFlagJob)

fun main() {
    if (isFeatureEnabled_SealedClass()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (isNotFeatureEnabled_SealedClass()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (featureService.isEnabled(ActivateStaleFlagJob)) {
        println("Hello world!")
    }
    if (featureService.isNotEnabled(ActivateStaleFlagJob)) {
        println("Hi world!")
    }
}

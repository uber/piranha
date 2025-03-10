import FeatureFlag.SomeStaleFeature

sealed class FeatureToggle(val name: String)

sealed class FeatureFlag(val name: String) {
    data object SomeStaleFeature : FeatureFlag("STALE_FLAG")
}

object ActivateFinCrimeChecks : FeatureToggle("should_activate_fin_crime_checks")
object ActivateSecondaryAccounts : FeatureToggle("should_activate_secondary_accounts")
object ActivateCarboniumConsumer : FeatureToggle("should_activate_carbonium_consumer")
object ActivateFailStuckPaymentsJob : FeatureToggle("should_activate_fail_stuck_payments_job")
object ActivateStaleFlagJob : FeatureToggle("STALE_FLAG")

object ObjectFeatures {
    const val AnotherFlag = "ANOTHER_FLAG"
    const val SomeStaleFalg = "STALE_FLAG"
}

sealed class ComplexFeatures(val flag: FeatureToggle, val something: String) {
    data object STALE_FLAG_COMPLEX : ComplexFeatures(ActivateFinCrimeChecks, "test")
    data object STALE_FLAG_COMPLEX_2 : ComplexFeatures(ActivateStaleFlagJob, "test")
}

sealed class ComplexFeatures2(val flag: ObjectFeatures, val something: String) {
    data object STALE_FLAG_COMPLEX : ComplexFeatures(AnotherFlag, "test")
    data object STALE_FLAG_COMPLEX_2 : ComplexFeatures(SomeStaleFalg, "test")
}

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

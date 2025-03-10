class FeatureFlagsTests {

    @Test
    fun test_referencing_old_flags_2() {
        STALE_FLAG
    }

    @Test
    fun test_referencing_old_flags() {
        val aFeature = STALE_FLAG

        every {
            featureClient.isEnabled(
                Feature(aFeature.name), false
            )
        } returns true

        assert(featureService.isFeatureEnabled(aFeature))
    }

    inner class SubTest {


        @BeforeEach
        fun setUp() {
            whenever(featureService.isFeatureEnabled(STALE_FLAG)).thenReturn(false)
        }

        @Test
        fun test_something() {
        }

        @Test
        fun test_something2() {
        }
    }
}

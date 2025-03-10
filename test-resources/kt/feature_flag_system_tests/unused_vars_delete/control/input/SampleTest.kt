class SampleTest {
    private val featureService: FeatureService = mockk()

    @Test
    fun testSomething() {
        val i = 0
        assertTrue(featureService.isEnabled(STALE_FLAG))
    }
}

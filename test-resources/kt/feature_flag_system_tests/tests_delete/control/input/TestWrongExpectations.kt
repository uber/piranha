class SampleTests {

    @Test
    fun testWheneverSomething() {
        whenever(featureService.isStaleFeatureEnabled()).thenReturn(false)
    }

    @Test
    fun testSomethingSetting() {
        stubService.setFeature(Feature(STALE_FLAG.name), false)
    }

    @Test
    fun testSomethingSetting2() {
        stubService.setFeature(Feature(STALE_FLAG.name), true)
    }

    @Test
    fun testSomethingSetting3() {
        stubService.setFeature(Feature(ANOTHER_FLAG.name), true)
    }

    @Test
    fun testWheneverSomething2() {
        every{ featureService.isStaleFeatureEnabled() } returns false
    }

    @Test
    fun testWheneverSomething3() {
        every
        { featureService.isStaleFeatureEnabled() } returns false
    }

    @Test
    fun testWheneverSomething4() {
        featureService.isStaleFeatureEnabled() shouldBe false
    }

    @Test
    fun testWheneverSomething5() {
        featureService.isStaleFeatureEnabled() shouldNotBe false
    }

    @Test
    fun testEnumCondition2() {
        featureService.isEnabled(STALE_FLAG) shouldBe false
    }
}

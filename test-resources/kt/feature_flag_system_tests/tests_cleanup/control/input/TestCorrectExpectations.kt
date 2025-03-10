class TestCorrectExpectations {

    @Test
    fun test1() {
        whenever(featureService.isStaleFeatureEnabled()).thenReturn(true)
        verify(featureService).isStaleFeatureEnabled()
        verify {
            someService.call()
            featureService.isStaleFeatureEnabled()
        }
        featureService.isStaleFeatureEnabled()
    }

    @Test
    fun test2() {
        print("ok")
        verify {
            featureService.isStaleFeatureEnabled()
        }
    }

    @Test
    fun test3() {
        print("ok")
    }
}

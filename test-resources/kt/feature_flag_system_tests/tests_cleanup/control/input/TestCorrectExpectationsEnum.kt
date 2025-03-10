class TestCorrectExpectations {

    @Test
    fun testEnumCondition() {
        whenever(featureService.isEnabled(STALE_FLAG)).thenReturn(true)
    }

    @Test
    fun testEnumCondition2() {
        featureService.isEnabled(STALE_FLAG) shouldBe true
    }

    @Test
    fun dummy() {
        print("ok")
    }
}

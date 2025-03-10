class TestCorrectExpectations {

    @Test
    @SuppressWarnings("UnusedPrivateProperty")
    fun test1() {
        verify {
            someService.call()
            true
        }
    }

    @Test
    @SuppressWarnings("UnusedPrivateProperty")
    fun test2() {
        print("ok")
        verify {
            true
        }
    }

    @Test
    fun test3() {
        print("ok")
    }
}

class TestCorrectExpectations {

    @Test
    fun test1() {
        verify {
            someService.call()
            true
        }
    }

    @Test
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

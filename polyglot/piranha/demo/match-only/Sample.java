public class Sample {


    public static void someMethodStatic(Foo foo) {
        System.out.println("This is a static method!");
        foo.fooBar("baz");
    }

    public void someMethodInstance(Foo foo) {
        System.out.println("This is a instance method!");
        foo.fooBar("baz");
    }
    
}

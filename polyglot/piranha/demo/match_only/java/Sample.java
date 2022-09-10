public class Sample {


    public static void someMethodStatic(Foo foo) {
        System.out.println("This is a static method!");
        foo.fooBar("baz");
    }

    public void someMethodInstance(Foo foo) {
        System.out.println("This is a instance method!");
        foo.fooBar("baz");
    }

    public static void someOtherMethodStatic(Foo foo) {
        System.out.println("This is a static method!");
        foo.barFoo("baz");
    }

    public void someOtherMethodInstance(Foo foo) {
        System.out.println("This is a instance method!");
        foo.barFoo("baz");
    }
    
}

package com.uber.piranha;

import com.uber.NewList;

class A {

    void foobar() {
        System.out.println("Hello World!");
        System.out.println(true);
    }
    
    @DoNotCleanup
    void barfn() {
        boolean b = foo().bar().baz();
        System.out.println(b);
    }

     void foofn() {
        int total = abc().fed().ghi();
    }

    void someTypeChange() {
        // Will get updated
        NewList a = getList();
        Integer item = getItem();
        a.addToNewList(item);

        // Will not get updated
        List<String> b = getListStr();
        Integer item = getItemStr();
        b.add(item);
    }

}

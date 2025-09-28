package com.uber.piranha;

import our.list.OurListOfInteger;
import our.map.OurMapOfStringInteger;
import our.map.OurMapOfLongFloat;
import not.our.map.NotOurMapOfDoubleInteger;

class A {

    void foobar() {
        // Will be removed
        boolean b = foo().bar().baz();
        if (b) {
            System.out.println("Hello World!");
        }
        System.out.println(b);
    }

    @DoNotCleanup
    void barfn() {
        boolean b = foo().bar().baz();
        System.out.println(b);
    }

    void foofn() {
        int total = abc().def().ghi();
    }

    void someTypeChange() {
        // Will get updated
        OurListOfInteger a = getList();
        Integer item = getItem();
        a.addToOurList(item);
        
        // Will not get updated
        List<String> b = getListStr();
        String item = getItemStr();
        b.add(item);
    }

    void someOtherTypeChange() {
        // Will get updated
        OurMapOfStringInteger siMap = getMapSI();
        String sKey = getStrKey();
        Integer iItem = getIItem();
        siMap.pushIntoOurMap(sKey, iItem);
        
        // Will get updated
        OurMapOfLongFloat lfMap = getMapLF();
        Long lKey = getLongKey();
        Float fItem = getFItem();
        lfMap.pushIntoOurMap(lKey, fItem);

        // Will not get updated
        NotOurMapOfDoubleInteger dlMap = getMapDL();
        Double dKey = getDoubleKey();
        dlMap.pushIntoOurMap(dKey, iItem);
    }

}

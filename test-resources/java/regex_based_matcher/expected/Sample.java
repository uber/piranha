package com.uber.piranha;

import java.util.HashMap;
import java.util.List;
import not.our.map.NotOurMapOfDoubleInteger;

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
        List<Integer> a = getList();
        Integer item = getItem();
        a.add(item);

        // Will not get updated
        List<String> b = getListStr();
        String item = getItemStr();
        b.add(item);
    }

    void someOtherTypeChange() {
        // Will get updated
        HashMap<String, Integer> siMap = getMapSI();
        String sKey = getStrKey();
        Integer iItem = getIItem();
        siMap.push(sKey, iItem);
        
        // Will get updated
        HashMap<Long, Float>  lfMap = getMapLF();
        Long lKey = getLongKey();
        Float fItem = getFItem();
        lfMap.push(lKey, fItem);

        // Will not get updated
        NotOurMapOfDoubleInteger dlMap = getMapDL();
        Double dKey = getDoubleKey();
        dlMap.pushIntoOurMap(dKey, iItem);
    }

}

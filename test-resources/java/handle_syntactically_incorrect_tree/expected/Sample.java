package com.uber.piranha;

import java.util.List;

class SomeClass {

    public static void run(List<Long> ls){        
        long sum = 0l;
        for (long l : ls){
            sum += l;
        }
    // Missing closing brace
}

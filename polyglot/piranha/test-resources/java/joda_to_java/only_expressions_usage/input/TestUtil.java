package com.uber.piranha;

import org.joda.time.DateTime;
import org.joda.time.format.DateTimeFormat;
import org.joda.time.format.DateTimeFormatter;


public class TestUtil {
    
    private static DateTimeFormatter dtfOut;
    private static long currentTime;
    

    public static void setUp(){

        dtfOut = DateTimeFormat.forPattern("yyyy/MM/dd");
        currentTime = new DateTime().toMillis();

    }

   

}

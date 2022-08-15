package com.uber.piranha;

import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import org.joda.time.DateTime;
import org.slf4j.Logger;

public class TestClass extends SomeClass{

  

    @Override
    public void window(String msg) {
        long start = new DateTime().getMillis();
        for (Map.Entry<String, Long> entry : cityTripTimeMap.entrySet()) {
            logger.info(" Trying to produce the trip data in the 10s window interval: " + entry);
            collector.send(new SendMessage(msg));
        }
        long end = new DateTime().getMillis();
        // does not have to be time.
        metric.update(end - start, TimeUnit.MILLISECONDS);
    }

   
}

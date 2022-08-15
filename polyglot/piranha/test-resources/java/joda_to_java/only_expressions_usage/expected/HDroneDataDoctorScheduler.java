package com.uber.piranha;

import java.time.Duration;

import java.util.Map;

public class Scheduler extends SomeC {
    private final Client client;
    private final DataSet datasets;

    public HDroneDataDoctorScheduler(DataSet datasets, Client client, Map<String, String> mapping) {
        super(Duration.ofHours(24), mapping);
        this.datasets = datasets;
        this.client = client;
    }
}

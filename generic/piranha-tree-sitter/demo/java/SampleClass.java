class SampleJava {

    public void sampleMethod(ExperimentInterface exp){
        if (exp.isToggleEnabled(SAMPLE_STALE_FLAG)){
            System.out.println("SAMPLE_STALE_FLAG is enabled");
        }else{
            System.out.println("SAMPLE_STALE_FLAG is disabled");
        }  
    }

    public void sampleMethod1(ExperimentInterface exp){
       if (exp.isToggleDisabled(ExpEnum.SAMPLE_STALE_FLAG)){
            System.out.println("SAMPLE_STALE_FLAG is disabled");
        }else{
            System.out.println("SAMPLE_STALE_FLAG is enabled");
        } 
    }
}
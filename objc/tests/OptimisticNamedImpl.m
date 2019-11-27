//  Based on the examples provided by Nicholas Lauer

#import <Foundation/Foundation.h>
#import <objc/runtime.h>


@protocol UBAdvancedExperimenting

- (BOOL)optimisticFeatureFlagEnabledForExperiment:(NSString *)experimentKey;

@end


#define UBOptimisticNamedFeatureFlagIsEnabled(flagName) ([UBOptimisticFeatureFlag isEnabled:@ #flagName])

#define UBOptimisticNamedFeatureFlag(flagName) \
    if (UBOptimisticNamedFeatureFlagIsEnabled(flagName))


@interface UBOptimisticFeatureFlag : NSObject
@property (class, nonatomic) id<UBAdvancedExperimenting> advancedExperiments;
+ (BOOL)isEnabled:(NSString *)flagName;
@end

@implementation UBOptimisticFeatureFlag

+ (BOOL)isEnabled:(NSString *)flagName
{
}

@end


@interface UBEOptimisticFeatureFlagCache : NSObject

+ (BOOL)optimistic_stale_flag;

@end

@implementation UBEOptimisticFeatureFlagCache

+ (BOOL)optimistic_stale_flag {
  static BOOL enabled = NO;
  static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        UBOptimisticNamedFeatureFlag(optimistic_stale_flag)
        {
            enabled = YES;
}
});
return enabled;
}


@implementation OptimisticTest

- (void)optimisticFeatureFlag_macro {
    if (UBEOptimisticFeatureFlagCache.optimistic_stale_flag) {
      NSLog(@"1");
    } else {
      NSLog(@"2");
    }
}

@end

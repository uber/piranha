/**
 *    Copyright (c) 2019 Uber Technologies, Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

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

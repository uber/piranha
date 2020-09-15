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

// swiftlint:disable custom_rules

import Foundation

protocol ExperimentKeying {
    var asString: String { get }
}

enum ExperimentNamesSwift: String, ExperimentKeying {
    //comment0
    case random_flag //comment1

    //comment4
    case test_experiment_suffix //comment5
    case test_second_experiment

    var asString: String {
        return String(describing: self)
    }
}

protocol TreatmentGroupKeying {
    var asString: String { get }
}

protocol ParamManager {
    func getParam(String param: String, defaultValue: Int) -> Bool
}

extension ParamManager {
    func getParam(String param: paramString, defaultValue: Int) -> Bool {
        return true
    }
}

protocol CachedExperimenting {
    func isInControlGroup(forExperiment experimentKey: ExperimentKeying) -> Bool
    func isTreated(forExperiment experimentKey: ExperimentKeying) -> Bool
    func addTreatedExperiment(forExperiment experimentKey: ExperimentKeying) -> Bool
    func removeTreatedExperiment(forExperiment experimentKey: ExperimentKeying) -> Bool
    func isInTreatmentGroup(treatmentGroup treatmentGroupKey: TreatmentGroupKeying, forExperiment experimentKey: ExperimentKeying) -> Bool
    func experimentParameter(name parameterName: String, forExperiment experimentKey: ExperimentKeying, defaultValue: String) -> String
    func experimentParameter(name parameterName: String, forExperiment experimentKey: ExperimentKeying, defaultValue: Int) -> Int
    func experimentParameter(name parameterName: String, forExperiment experimentKey: ExperimentKeying, defaultValue: Float) -> Float
    func experimentParameter(name parameterName: String, forExperiment experimentKey: ExperimentKeying, defaultValue: Double) -> Double
    func sendInclusionEvent(forExperiment experimentKey: ExperimentKeying, treatmentGroup treatmentGroupKey: TreatmentGroupKeying)
}

extension CachedExperimenting {

    func isInControlGroup(forExperiment experimentKey: ExperimentKeying) -> Bool {
        return true
    }

    func isTreated(forExperiment experimentKey: ExperimentKeying) -> Bool {
        return true
    }

    func isInTreatmentGroup(treatmentGroup treatmentGroupKey: TreatmentGroupKeying, forExperiment experimentKey: ExperimentKeying) -> Bool {
        return true
    }

    func experimentParameter(name parameterName: String, forExperiment experimentKey: ExperimentKeying, defaultValue: String) -> String {
        return "foobar"
    }

    func experimentParameter(name parameterName: String, forExperiment experimentKey: ExperimentKeying, defaultValue: Int) -> Int {
        return 42
    }

    func experimentParameter(name parameterName: String, forExperiment experimentKey: ExperimentKeying, defaultValue: Float) -> Float {
        return 42.0
    }

    func experimentParameter(name parameterName: String, forExperiment experimentKey: ExperimentKeying, defaultValue: Double) -> Double {
        return 42.0
    }

    func sendInclusionEvent(forExperiment experimentKey: ExperimentKeying, treatmentGroup treatmentGroupKey: TreatmentGroupKeying) {
    }
}

class CachedExperiments: CachedExperimenting {

    init() {
    }

}

class SwiftExamples {

    private let impressionStr: String

    let cachedExperiments = CachedExperiments()

    public enum test_12experiment: String {
        case delay
    }

    let p1 = "p1"
    static var p2 = "p2"

    private let fieldZ: Bool

    func test_expressions() {

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment_suffix) {
            print("treated")
        }
        print("not treated / control")

        let x = false
        var y = false

        if x {
            print("test 1")
        }

        if x || y {
            print("test 11")
        }

        y = false
        y = false
        y = x
        print("1")
        print("not treated / control")
        return
    }

    func addTreatedExperiment(forExperiment experimentKey: ExperimentKeying) -> Bool {
        return true

    }

    func test_additional() -> Bool {

        var platformUIChange = true
        var recordMode = true
        recordMode = false || platformUIChange

        recordMode = platformUIChange
        recordMode = recordMode

        recordMode = platformUIChange == recordMode ? platformUIChange : recordMode
        print("not treated / control")

        if recordMode {
        } else {

        }

        print("not treated / control")

        return false
    }

    func test_closure() -> Bool {

        cachedExperimentingMock.isTreatedHandler = { (key: ExperimentKeying) in
            return key.asString == ExperimentNamesSwift.test_second_experiment.asString
        }
        return true

    }

    // test for T2205641
    func test_experimentParameter() -> Bool {
        return true
    }

    func test_T2206585() -> Bool {
        print("br2")
        print("br4")
        print("br6")
        print("br8")

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_second_experiment) {
            print("br9")
        } else {
            print("br10")
        }

        return true

    }

    // tests for task T2191251
    func storeuse_before() {
        print("somestring2")
        print("somestring4")

        if self.fieldZ {
            print("fieldXfieldYfieldZ")
        }

        if fieldZ {
            print("pqr")
        }
    }

    func storeuse_init() {
        fieldZ = cachedexperiments.isTreated(forexperiment: ExperimentNamesSwift.test_experiment2)
    }

    func storeuse_after() {
        print("somestring2")
        print("somestring4")

        if self.fieldZ {
            print("XYZ")
        }

        if fieldZ {
            print("pqr")
        }
    }

    func test_nilcoalescing() {
         print("control1")
         print("treated2")
         print("treated3")
         print("control4")
         print("control5")
         print("treated6")

        if cachedExperiments?.isTreated(forExperiment: ExperimentNamesSwift.test_second_experiment) ?? false {
            print("treated1")
        } else {
            print("control1")
        }

        v1 = true

        v2 = cachedExperiments?.isInControlGroup(forExperiment: ExperimentNamesSwift.test_second_experiment) ?? false
    }

    private let conj1: Bool
    private let conj2: Bool
    private let conj3: Bool

    func test_T2282603() {
        self.conj1 = cachedexperiments.isInControlGroup(forexperiment: ExperimentNamesSwift.test_second_experiment)
        self.conj2 = false
        self.conj3 = cachedexperiments.isInControlGroup(forexperiment: ExperimentNamesSwift.test_experiment_suffix)

    }

    // Test for T2606011
    private var shouldDoSomething: Bool {
        return false
    }

    private let engineeringFlags: [ExperimentNamesLoyalty] = [
        .loyalty_credits_purchase_selection_rib_refactor,
        .loyalty_card_banner_impression_fix,
        .loyalty_credits_purchase_selection_default_payment_profile_fix,
        .loyalty_credits_purchase_addon_explicit_layout,
        .loyalty_stack_view_migration
    ]
}

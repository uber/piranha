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

    ///comment2
    case test_experiment // comment3

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
    func isTreated(for experimentKey: String) -> Bool
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

    public enum test_experiment: String, TreatmentGroupKeying {
        case delay
    }

    public enum test_12experiment: String {
        case delay
    }

    let p1 = "p1"
    static var p2 = "p2"

    let fieldX: Bool
    private let fieldY: Bool
    private let fieldZ: Bool

    private lazy var fieldA: Bool = !cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment)

    func test_expressions() {

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) {
            print("treated")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment_suffix) {
            print("treated")
        }

        if !cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) {
            print("not treated / control")
        }

        let x = false
        var y = false


        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) || x {
            print("test 1")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) && x {
            print("test 2")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment), x {
            print("test 3")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) && (x || y) {
            print("test 4")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment), (x || y) {
            print("test 5")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) && (x && y) {
            print("test 6")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment), (x && y) {
            print("test 7")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment), x, y {
            print("test 8")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) && y == x {
            print("test 9")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment), y == x {
            print("test 10")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) || x || y {
            print("test 11")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) && x && y {
            print("test 12")
        }

        y = cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment)
        y = cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) && x
        y = cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) || x
        
        if fieldA {
            print("1")
        } else {
            print("2")
        }

        guard cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) else {
            print("not treated / control")
            return
        }

        print("treated")

        while true {

        }

        var xyz = getParam("hello")

        let abc = "world"

        xyz = getParam(abc)
        getParam(p1)
        getParam(p2)
        getParam(xyz)
    }


    func addTreatedExperiment(forExperiment experimentKey: ExperimentKeying) -> Bool {
        return true

    }

    func test_additional() -> Bool {

        var platformUIChange = true
        var recordMode = true
        recordMode = false || platformUIChange

        recordMode = cachedExperiments.isInControlGroup(forExperiment: ExperimentNamesSwift.test_experiment) ? platformUIChange : recordMode
        recordMode = cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) ? platformUIChange : recordMode

        recordMode = platformUIChange == recordMode ? platformUIChange : recordMode

        if cachedExperiments.isInControlGroup(forExperiment: ExperimentNamesSwift.test_experiment) {
            print("not treated / control")
        }

        if !cachedExperiments.isInControlGroup(forExperiment: ExperimentNamesSwift.test_experiment) {
            print("treated")
        }

        guard cachedExperiments.isInControlGroup(forExperiment: ExperimentNamesSwift.test_experiment) else {
            print("treated")
            return
        }

        if recordMode {
            addTreatedExperiment(forExperiment: ExperimentNamesSwift.test_experiment)
        } else {
            cachedExperiments.removeTreatedExperiment(forExperiment: ExperimentNamesSwift.test_experiment)

        }

        print("not treated / control")

        return false && recordMode == platformUIChange
    }

    func test_closure() -> Bool {
        cachedExperimentingMock.isTreatedHandler = { (key: ExperimentKeying) in
            return key.asString == ExperimentNamesSwift.test_experiment.asString
        }

        cachedExperimentingMock.isTreatedHandler = { (key: ExperimentKeying) in
            return key.asString == ExperimentNamesSwift.test_second_experiment.asString
        }

        cachedExperimentingMock.isTreatedHandler2 = { (key: ExperimentKeying) in
            return key.asString == ExperimentNamesSwift.test_experiment.asString ||
                key.asString == ExperimentNamesSwift.test_second_experiment.asString

        }

        cachedExperiments.isTreatedHandler = { (_ experiment: ExperimentKeying) in
            if experiment.asString == ExperimentNamesSwift.test_experiment.asString {
                return true
            }
            return false
        }
        return true

    }

    // test for T2205641
    func test_experimentParameter() -> Bool {
        cachedExperiments.experimentParameter("abcd", ExperimentNamesSwift.test_experiment, "efgh")
        self.impressionStr = cachedExperiments.experimentParameter("abcd", ExperimentNamesSwift.test_experiment, "efgh")
        return true
    }

    func test_T2206585() -> Bool {

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_second_experiment)
            && cachedExperiments.isTreated(forExperiment: ExperimentNamesRewards.test_experiment)
            && self.impressionStr != nil {
            print("br1")
        } else {
            print("br2")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment)
            && cachedExperiments.isTreated(forExperiment: ExperimentNamesRewards.test_second_experiment)
            && self.impressionStr != nil {
            print("br3")
        } else {
            print("br4")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_second_experiment)
            && self.impressionStr != nil
            && cachedExperiments.isTreated(forExperiment: ExperimentNamesRewards.test_experiment) {
            print("br5")
        } else {
            print("br6")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_second_experiment)
            && self.impressionStr != nil
            && self.impressionStr == "abcd"
            && cachedExperiments.isTreated(forExperiment: ExperimentNamesRewards.test_experiment) {
            print("br7")
        } else {
            print("br8")
        }

        if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_second_experiment)
            || (cachedExperiments.isTreated(forExperiment: ExperimentNamesRewards.test_experiment)
                && self.impressionStr != nil) {
            print("br9")
        } else {
            print("br10")
        }

        return true

    }

    // tests for task T2191251
    func storeuse_before() {
        if self.fieldX {
            print("Hi world")
        }

        if fieldX {
            print("somestring0")
        }

        if fieldY {
            print("somestring1")
        }

        if self.fieldY {
            print("randomstring")
        }

        if !fieldY {
            print("somestring2")
        } else {
            print("somestring3")
        }

        if !self.fieldY {
            print("somestring4")
        } else {
            print("somestring5")
        }

        if self.fieldY && fieldZ {
            print("somestring6")
        }

        if self.fieldZ {
            print("fieldXfieldYfieldZ")
        }

        if fieldZ {
            print("pqr")
        }
    }

    func storeuse_init() {

        self.fieldX = cachedexperiments.isTreated(forexperiment: ExperimentNamesSwift.test_experiment)
        fieldY = cachedexperiments.isTreated(forexperiment: ExperimentNamesSwift.test_experiment)
        fieldZ = cachedexperiments.isTreated(forexperiment: ExperimentNamesSwift.test_experiment2)

        if self.fieldX {
            print("Hello world")
        }

        if self.fieldY {
            print("Hi world")
        }
    }

    func storeuse_after() {
        if self.fieldX {
            print("Hi world")
        }

        if fieldX {
            print("somestring0")
        }

        if fieldY {
            print("somestring1")
        }

        if self.fieldY {
            print("randomstring")
        }

        if !fieldY {
            print("somestring2")
        } else {
            print("somestring3")
        }

        if !self.fieldY {
            print("somestring4")
        } else {
            print("somestring5")
        }

        if self.fieldY && fieldZ {
            print("somestring6")
        }

        if self.fieldZ {
            print("XYZ")
        }

        if fieldZ {
            print("pqr")
        }
    }

    func test_nilcoalescing() {

        if cachedExperiments?.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) ?? false {
            print("treated1")
        } else {
            print("control1")
        }

        if cachedExperiments?.isInControlGroup(forExperiment: ExperimentNamesSwift.test_experiment) ?? false {
            print("treated2")
        } else {
            print("control2")
        }

        if (!cachedExperiments?.isTreated(forExperiment: ExperimentNamesSwift.test_experiment)) ?? false {
            print("treated3")
        } else {
            print("control3")
        }

        if !(cachedExperiments?.isInControlGroup(forExperiment: ExperimentNamesSwift.test_experiment)) ?? false {
            print("treated4")
        } else {
            print("control4")
        }

        if cachedExperiments?.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) ?? (cachedExperiments?.isTreated(forExperiment: ExperimentNamesSwift.test_second_experiment) ?? false) {
            print("treated5")
        } else {
            print("control5")
        }

        if !cachedExperiments?.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) ?? (cachedExperiments?.isTreated(forExperiment: ExperimentNamesSwift.test_second_experiment) ?? false) {
            print("treated6")
        } else {
            print("control6")
        }

        if cachedExperiments?.isTreated(forExperiment: ExperimentNamesSwift.test_second_experiment) ?? false {
            print("treated1")
        } else {
            print("control1")
        }

        var v = cachedExperiments?.isTreated(forExperiment: ExperimentNamesSwift.test_experiment) ?? false
        v1 = cachedExperiments?.isInControlGroup(forExperiment: ExperimentNamesSwift.test_experiment) ?? false
        v2 = cachedExperiments?.isInControlGroup(forExperiment: ExperimentNamesSwift.test_second_experiment) ?? false
    }

    private let conj1: Bool
    private let conj2: Bool
    private let conj3: Bool

    func test_T2282603() {
        self.conj1 = cachedexperiments.isInControlGroup(forexperiment: ExperimentNamesSwift.test_second_experiment)
        self.conj2 = cachedexperiments.isInControlGroup(forexperiment: ExperimentNamesSwift.test_experiment_suffix) &&
                        cachedexperiments.isTreated(forexperiment: ExperimentNamesSwift.test_experiment)

        self.conj3 = cachedexperiments.isTreated(forexperiment: ExperimentNamesSwift.test_experiment) || cachedexperiments.isInControlGroup(forexperiment: ExperimentNamesSwift.test_experiment_suffix) 
    }

    // Test for T2606011
    private var shouldDoSomething: Bool {
        return cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.test_experiment)
    }


    func testStringFlag() {
        if cachedExperiments.isTreated(for: "test_experiment") {
           print("string constant 1")
        }

        if !cachedExperiments.isTreated(for: "test_experiment") {
           print("string constant 2")
        }

    }
    
    private let engineeringFlags: [ExperimentNamesLoyalty] = [
        .loyalty_credits_purchase_selection_rib_refactor,
        .loyalty_card_banner_impression_fix,
        .loyalty_credits_purchase_selection_default_payment_profile_fix,
        .test_experiment,
        .loyalty_credits_purchase_addon_explicit_layout,
        .loyalty_stack_view_migration
    ]
}

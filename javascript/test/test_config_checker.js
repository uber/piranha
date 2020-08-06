const chai = require("chai");
const expect = chai.expect;
const config_checker = require("../src/config_checker");
const parseProperties = config_checker.parseProperties;

describe("config_checker", () => {
  describe("#parseProperties", () => {
    it("should throw error on invalid properties filepath.", () => {
      expect(
        parseProperties.bind(
          parseProperties,
          "random_prop_file_that_does_not_exist.json",
          true
        )
      ).to.throw("File random_prop_file_that_does_not_exist.json not found");
    });

    it("should throw error on invalid json syntax.", () => {
      expect(
        parseProperties.bind(
          parseProperties,
          "test/wrong_properties/wrong_properties1.json",
          true
        )
      ).to.throw(
        "test/wrong_properties/wrong_properties1.json does not follow JSON syntax"
      );
    });

    it("should throw error on missing keys in some method.", () => {
      expect(
        parseProperties.bind(
          parseProperties,
          "test/wrong_properties/wrong_properties2.json",
          true
        )
      ).to.throw(
        `{"methodName":"isToggleEnabled","flagType":"treated"} in test/wrong_properties/wrong_properties2.json doesn't have all required keys`
      );
    });

    it("should throw error on missing key methodProperties.", () => {
      expect(
        parseProperties.bind(
          parseProperties,
          "test/wrong_properties/wrong_properties3.json",
          true
        )
      ).to.throw(
        "The methodProperties property is missing in test/wrong_properties/wrong_properties3.json"
      );
    });
  });
});

const chai = require('chai');
const expect = chai.expect;
const source_checker = require('../src/source_checker');
const checkSource = source_checker.checkSource;

describe('source_checker', () => {
    describe('#checkSource', () => {
        it("should throw error on invalid source filepath.", () => {
           expect(checkSource.bind(checkSource, 'random_source_file_that_does_not_exist.js', true))
           .to.throw('File random_source_file_that_does_not_exist.js not found');
        });
    
        it("should throw error if source file is not JS.", () => {
           // fs module uses process.cwd() to figure relative path so `npm test` run in root wont't need ../ 
           expect(checkSource.bind(checkSource, 'config/properties.json', true))
           .to.throw('Input config/properties.json is not a javascript file');
        });
     });
})

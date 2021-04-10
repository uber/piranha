//
//  File.swift
//  
//
//  Created by Chirag Ramani on 10/04/21.
//

import Foundation
import ArgumentParser

struct CleanupStaleFlagsCommandInputValidator {
    
    func validateInput(sourceFileURL: URL?,
                       configFileURL: URL?,
                       flag: String,
                       fileManager: FileManager) throws {
        guard let sourceFileURL = sourceFileURL,
              fileManager.fileExists(atPath: sourceFileURL.path) else {
            throw ValidationError("Please provide valid source file path")
        }
        
        guard let configFileURL = configFileURL,
              fileManager.fileExists(atPath: configFileURL.path) else {
            throw ValidationError("Please provide valid config file path")
        }
        
        
        guard !flag.isEmpty else {
            throw ValidationError("Please provide valid flag name")
        }
    }
}

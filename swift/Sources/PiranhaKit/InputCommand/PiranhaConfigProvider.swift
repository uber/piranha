//
//  File.swift
//  
//
//  Created by Chirag Ramani on 15/04/21.
//

import ArgumentParser
import Foundation

protocol PiranhaConfigProviding {
    func config(fromFileAtURL url: URL) throws -> PiranhaConfig
}

struct PiranhaConfigProviderDefaultImpl: PiranhaConfigProviding {
    
    func config(fromFileAtURL url: URL) throws -> PiranhaConfig {
        do {
            let properties = try Data(contentsOf: url)
            return try JSONDecoder().decode(PiranhaConfig.self,
                                            from: properties)
        } catch {
            throw ValidationError("Invalid configuration")
        }
    }
}

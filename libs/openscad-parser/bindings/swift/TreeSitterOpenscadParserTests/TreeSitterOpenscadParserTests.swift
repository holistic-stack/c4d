import XCTest
import SwiftTreeSitter
import TreeSitterOpenscadParser

final class TreeSitterOpenscadParserTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_openscad_parser())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading OpenscadParser grammar")
    }
}

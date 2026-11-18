port module Backend exposing
    ( AutomatonRun
    , ClosedFormComparison
    , DrawnPolycube
    , EnumerationStep
    , ExposedFaceHistograms
    , ExtensionSize
    , Gallery
    , GrownPolycube
    , GrowthComparison
    , Pattern
    , PatternRule
    , RemarkablePolycubes
    , RemarkableShape
    , ShapeFamily
    , automatonRunFinished
    , closedFormComparisonFinished
    , drawFinished
    , enumerationTraceFinished
    , galleryFinished
    , growthComparisonFinished
    , patternRunFinished
    , patternsFinished
    , ready
    , remarkableFinished
    , requestAutomatonRun
    , requestClosedFormComparison
    , requestDraw
    , requestEnumerationTrace
    , requestGallery
    , requestGrowthComparison
    , requestPatternRun
    , requestPatterns
    , requestRemarkable
    )

{-| Calls to the Rust backend, relayed by `public/backend.js`.
-}

import Json.Decode as Decode exposing (Decoder)
import Json.Encode as Encode
import Polycube exposing (Cell, Polycube)


{-| A polycube drawn uniformly among the fixed polycubes of its size.

Rank and count are text: they may exceed what a JavaScript number holds.

-}
type alias DrawnPolycube =
    { rank : String
    , polycubeCount : String
    , cells : Polycube
    }


{-| A polycube whose cells are listed in the order they appear, with the
side lengths of the smallest box holding it.
-}
type alias GrownPolycube =
    { cells : Polycube
    , boundingBox : List Int
    }


{-| Two polycubes of the same size: one grown by picking a free neighbour at
random, one drawn uniformly.
-}
type alias GrowthComparison =
    { neighbourGrowth : GrownPolycube
    , uniformDraw : GrownPolycube
    }


{-| One polycube met by the enumeration, with what the enumeration holds
when it reaches it: the cells waiting to extend it, the cells it may no
longer hold, and how many polycubes of each size were found so far.
-}
type alias EnumerationStep =
    { placed : Polycube
    , candidates : List Cell
    , discarded : List Cell
    , countsBySize : List Int
    }


{-| A polycube met by the enumeration, the cells waiting to extend it, and
its extensions: entry `added - 1` holds those that add `added` cells.
-}
type alias ClosedFormComparison =
    { placed : Polycube
    , candidates : List Cell
    , extensionSizes : List ExtensionSize
    }


{-| The extensions adding a given number of cells, counted by the closed
formula and by visiting them. Both counts are split by how many added cells
lie beyond the waiting cells; `examples` holds some of the visited ones.
-}
type alias ExtensionSize =
    { formulaCounts : List Int
    , visitedCounts : List Int
    , examples : List (List Cell)
    }


{-| A sample of uniform draws and one of polycubes grown by random
neighbour. Each histogram counts the polycubes per tenth of fill rate of the
bounding box, from the emptiest boxes to the fullest; `shown` holds some of
the uniform draws, from the most compact to the least.
-}
type alias Gallery =
    { shown : List GrownPolycube
    , uniformHistogram : List Int
    , growthHistogram : List Int
    , exposedFaces : ExposedFaceHistograms
    , sampleCount : Int
    }


{-| The same two samples counted by number of exposed faces: entry `i`
counts the polycubes showing `smallestFaceCount + 2 * i` faces. The last
entry is the most faces a polycube of that size can show, reached when it
has no loop.
-}
type alias ExposedFaceHistograms =
    { smallestFaceCount : Int
    , uniform : List Int
    , growth : List Int
    }


{-| Three families of exceptional polycubes of one size: those with the
most loops, those with the most loops among the polycubes holding no 2 × 2
square (absent when none has a loop), and the trees fitting in the smallest
box.
-}
type alias RemarkablePolycubes =
    { mostLooped : ShapeFamily
    , squareFreeLoops : Maybe ShapeFamily
    , tightestTrees : ShapeFamily
    }


{-| Shapes sharing an extreme. `shown` holds the most symmetric ones; each
shape stands for `orientationCount` fixed polycubes, itself turned around.
-}
type alias ShapeFamily =
    { loopCount : Int
    , exposedFaceCount : Int
    , shapeCount : Int
    , fixedCount : Int
    , shown : List RemarkableShape
    }


type alias RemarkableShape =
    { polycube : GrownPolycube
    , orientationCount : Int
    }


{-| The generations of a polycube that grows, the seed first, and two
opposite corners of the box holding them all. `parentOfCopy` is filled when
the generations list polycubes that bore one another: the birth rank of the
parent of each. `identitiesByGeneration` is filled when polycubes die: for
each generation, what tells each of its polycubes from all others ; and
`shapesByGeneration` with it: a number shared by all polycubes of one shape.
-}
type alias AutomatonRun =
    { generations : List (List Cell)
    , lowestCorner : Cell
    , highestCorner : Cell
    , parentOfCopy : List Int
    , identitiesByGeneration : List (List Int)
    , shapesByGeneration : List (List Int)
    }


{-| Fires once `public/backend.js` listens to the requests. A request sent
earlier, from `init` for instance, would be lost.
-}
port ready : (() -> msg) -> Sub msg


{-| Asks for a uniform draw among the fixed polycubes of the given size.
-}
port requestDraw : Int -> Cmd msg


port drawFinishedPort : (Decode.Value -> msg) -> Sub msg


{-| Asks for a growth comparison between polycubes of the given size.
-}
port requestGrowthComparison : Int -> Cmd msg


port growthComparisonFinishedPort : (Decode.Value -> msg) -> Sub msg


{-| Asks for every step of the enumeration of the polycubes of at most the
given size.
-}
port requestEnumerationTrace : Int -> Cmd msg


port enumerationTraceFinishedPort : (Decode.Value -> msg) -> Sub msg


enumerationTraceFinished : (Result String (List EnumerationStep) -> msg) -> Sub msg
enumerationTraceFinished toMsg =
    enumerationTraceFinishedPort (decodeResult (Decode.list enumerationStepDecoder) >> toMsg)


{-| Asks for a polycube of the given size and both counts of its
extensions.
-}
port requestClosedFormComparison : Int -> Cmd msg


port closedFormComparisonFinishedPort : (Decode.Value -> msg) -> Sub msg


closedFormComparisonFinished : (Result String ClosedFormComparison -> msg) -> Sub msg
closedFormComparisonFinished toMsg =
    closedFormComparisonFinishedPort (decodeResult closedFormComparisonDecoder >> toMsg)


{-| Asks for a gallery of polycubes of the given size.
-}
port requestGallery : Int -> Cmd msg


port galleryFinishedPort : (Decode.Value -> msg) -> Sub msg


galleryFinished : (Result String Gallery -> msg) -> Sub msg
galleryFinished toMsg =
    galleryFinishedPort (decodeResult galleryDecoder >> toMsg)


{-| Asks for the exceptional polycubes of the given size.
-}
port requestRemarkable : Int -> Cmd msg


port remarkableFinishedPort : (Decode.Value -> msg) -> Sub msg


remarkableFinished : (Result String RemarkablePolycubes -> msg) -> Sub msg
remarkableFinished toMsg =
    remarkableFinishedPort (decodeResult remarkableDecoder >> toMsg)


drawFinished : (Result String DrawnPolycube -> msg) -> Sub msg
drawFinished toMsg =
    drawFinishedPort (decodeResult drawnPolycubeDecoder >> toMsg)


growthComparisonFinished : (Result String GrowthComparison -> msg) -> Sub msg
growthComparisonFinished toMsg =
    growthComparisonFinishedPort (decodeResult growthComparisonDecoder >> toMsg)


decodeResult : Decoder value -> Decode.Value -> Result String value
decodeResult valueDecoder json =
    let
        resultDecoder =
            Decode.oneOf
                [ Decode.field "ok" valueDecoder |> Decode.map Ok
                , Decode.field "error" Decode.string |> Decode.map Err
                ]
    in
    case Decode.decodeValue resultDecoder json of
        Ok result ->
            result

        Err decodeError ->
            Err (Decode.errorToString decodeError)


drawnPolycubeDecoder : Decoder DrawnPolycube
drawnPolycubeDecoder =
    Decode.map3 DrawnPolycube
        (Decode.field "rank" Decode.string)
        (Decode.field "polycubeCount" Decode.string)
        (Decode.field "cells" (Decode.list Polycube.cellDecoder))


growthComparisonDecoder : Decoder GrowthComparison
growthComparisonDecoder =
    Decode.map2 GrowthComparison
        (Decode.field "neighbourGrowth" grownPolycubeDecoder)
        (Decode.field "uniformDraw" grownPolycubeDecoder)


grownPolycubeDecoder : Decoder GrownPolycube
grownPolycubeDecoder =
    Decode.map2 GrownPolycube
        (Decode.field "cells" (Decode.list Polycube.cellDecoder))
        (Decode.field "boundingBox" (Decode.list Decode.int))


enumerationStepDecoder : Decoder EnumerationStep
enumerationStepDecoder =
    Decode.map4 EnumerationStep
        (Decode.field "placed" cellsDecoder)
        (Decode.field "candidates" cellsDecoder)
        (Decode.field "discarded" cellsDecoder)
        (Decode.field "countsBySize" (Decode.list Decode.int))


closedFormComparisonDecoder : Decoder ClosedFormComparison
closedFormComparisonDecoder =
    Decode.map3 ClosedFormComparison
        (Decode.field "placed" cellsDecoder)
        (Decode.field "candidates" cellsDecoder)
        (Decode.field "extensionSizes" (Decode.list extensionSizeDecoder))


extensionSizeDecoder : Decoder ExtensionSize
extensionSizeDecoder =
    Decode.map3 ExtensionSize
        (Decode.field "formulaCounts" (Decode.list Decode.int))
        (Decode.field "visitedCounts" (Decode.list Decode.int))
        (Decode.field "examples" (Decode.list cellsDecoder))


cellsDecoder : Decoder (List Cell)
cellsDecoder =
    Decode.list Polycube.cellDecoder


galleryDecoder : Decoder Gallery
galleryDecoder =
    Decode.map5 Gallery
        (Decode.field "shown" (Decode.list grownPolycubeDecoder))
        (Decode.field "uniformHistogram" (Decode.list Decode.int))
        (Decode.field "growthHistogram" (Decode.list Decode.int))
        (Decode.field "exposedFaces" exposedFaceHistogramsDecoder)
        (Decode.field "sampleCount" Decode.int)


exposedFaceHistogramsDecoder : Decoder ExposedFaceHistograms
exposedFaceHistogramsDecoder =
    Decode.map3 ExposedFaceHistograms
        (Decode.field "smallestFaceCount" Decode.int)
        (Decode.field "uniform" (Decode.list Decode.int))
        (Decode.field "growth" (Decode.list Decode.int))


remarkableDecoder : Decoder RemarkablePolycubes
remarkableDecoder =
    Decode.map3 RemarkablePolycubes
        (Decode.field "mostLooped" shapeFamilyDecoder)
        (Decode.field "squareFreeLoops" (Decode.nullable shapeFamilyDecoder))
        (Decode.field "tightestTrees" shapeFamilyDecoder)


shapeFamilyDecoder : Decoder ShapeFamily
shapeFamilyDecoder =
    Decode.map5 ShapeFamily
        (Decode.field "loopCount" Decode.int)
        (Decode.field "exposedFaceCount" Decode.int)
        (Decode.field "shapeCount" Decode.int)
        (Decode.field "fixedCount" Decode.int)
        (Decode.field "shown" (Decode.list remarkableShapeDecoder))


remarkableShapeDecoder : Decoder RemarkableShape
remarkableShapeDecoder =
    Decode.map2 RemarkableShape
        (Decode.field "polycube" grownPolycubeDecoder)
        (Decode.field "orientationCount" Decode.int)


{-| Asks for a seed and its generations. The request holds `cellCount`,
`seedKind` and `mode`, as the backend names them.
-}
port requestAutomatonRunPort : Encode.Value -> Cmd msg


port automatonRunFinishedPort : (Decode.Value -> msg) -> Sub msg


requestAutomatonRun :
    { cellCount : Int, seedKind : String, mode : String, aggregateCellCount : Int, lifeWorldReach : Int, conwayStart : String }
    -> Cmd msg
requestAutomatonRun request =
    requestAutomatonRunPort
        (Encode.object
            [ ( "cellCount", Encode.int request.cellCount )
            , ( "seedKind", Encode.string request.seedKind )
            , ( "mode", Encode.string request.mode )
            , ( "aggregateCellCount", Encode.int request.aggregateCellCount )
            , ( "lifeWorldReach", Encode.int request.lifeWorldReach )
            , ( "conwayStart", Encode.string request.conwayStart )
            ]
        )


automatonRunFinished : (Result String AutomatonRun -> msg) -> Sub msg
automatonRunFinished toMsg =
    automatonRunFinishedPort (decodeResult automatonRunDecoder >> toMsg)


automatonRunDecoder : Decoder AutomatonRun
automatonRunDecoder =
    Decode.map6 AutomatonRun
        (Decode.field "generations" (Decode.list cellsDecoder))
        (Decode.field "lowestCorner" Polycube.cellDecoder)
        (Decode.field "highestCorner" Polycube.cellDecoder)
        (Decode.field "parentOfCopy" (Decode.list Decode.int))
        (Decode.field "identitiesByGeneration" (Decode.list (Decode.list Decode.int)))
        (Decode.field "shapesByGeneration" (Decode.list (Decode.list Decode.int)))


{-| A pattern of the gallery of the Conway-like life, its lowest corner at
the origin. `nature` is `still`, `oscillator` or `glider`; `symmetry` the
number of the 24 rotations of the cube that leave it as it was.
-}
type alias Pattern =
    { number : Int
    , shape : String
    , nature : String
    , period : Int
    , symmetry : Int
    , rule : PatternRule
    , polycubes : List (List Cell)
    }


{-| The rule a pattern lives under.
-}
type alias PatternRule =
    { fewestToSurvive : Int
    , mostToSurvive : Int
    , fewestToBeBorn : Int
    , mostToBeBorn : Int
    , fewestFacesShared : Int
    }


{-| Asks for the patterns of the gallery.
-}
port requestPatternsPort : Encode.Value -> Cmd msg


port patternsFinishedPort : (Decode.Value -> msg) -> Sub msg


requestPatterns : Cmd msg
requestPatterns =
    requestPatternsPort (Encode.object [])


patternsFinished : (Result String (List Pattern) -> msg) -> Sub msg
patternsFinished toMsg =
    patternsFinishedPort (decodeResult (Decode.list patternDecoder) >> toMsg)


{-| Asks for the generations of the pattern bearing the given number,
played under its own rule.
-}
port requestPatternRunPort : Encode.Value -> Cmd msg


port patternRunFinishedPort : (Decode.Value -> msg) -> Sub msg


requestPatternRun : Int -> Cmd msg
requestPatternRun number =
    requestPatternRunPort (Encode.object [ ( "number", Encode.int number ) ])


patternRunFinished : (Result String AutomatonRun -> msg) -> Sub msg
patternRunFinished toMsg =
    patternRunFinishedPort (decodeResult automatonRunDecoder >> toMsg)


patternDecoder : Decoder Pattern
patternDecoder =
    Decode.map7 Pattern
        (Decode.field "number" Decode.int)
        (Decode.field "shape" Decode.string)
        (Decode.field "nature" Decode.string)
        (Decode.field "period" Decode.int)
        (Decode.field "symmetry" Decode.int)
        (Decode.field "rule" patternRuleDecoder)
        (Decode.field "polycubes" (Decode.list cellsDecoder))


patternRuleDecoder : Decoder PatternRule
patternRuleDecoder =
    Decode.map5 PatternRule
        (Decode.field "fewest_neighbours_to_survive" Decode.int)
        (Decode.field "most_neighbours_to_survive" Decode.int)
        (Decode.field "fewest_neighbours_to_be_born" Decode.int)
        (Decode.field "most_neighbours_to_be_born" Decode.int)
        (Decode.field "fewest_faces_shared_to_be_born" Decode.int)

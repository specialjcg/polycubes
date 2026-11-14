port module Backend exposing
    ( AutomatonCensus
    , AutomatonRun
    , ClosedFormComparison
    , DrawnPolycube
    , ExposedFaceHistograms
    , ExtensionSize
    , Fate(..)
    , Gallery
    , EnumerationStep
    , GrownPolycube
    , GrowthComparison
    , RemarkablePolycubes
    , RemarkableShape
    , ShapeFamily
    , automatonCensusFinished
    , automatonRunFinished
    , closedFormComparisonFinished
    , drawFinished
    , enumerationTraceFinished
    , galleryFinished
    , growthComparisonFinished
    , remarkableFinished
    , ready
    , requestAutomatonCensus
    , requestAutomatonRun
    , requestClosedFormComparison
    , requestDraw
    , requestEnumerationTrace
    , requestGallery
    , requestGrowthComparison
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


{-| The generations of a cellular automaton, the seed first, and what
becomes of that seed.
-}
type alias AutomatonRun =
    { generations : List (List Cell)
    , fate : Fate
    }


{-| The integer is the generation where the final behaviour starts; cycles
and travels also carry their period.
-}
type Fate
    = Dies Int
    | Still Int
    | Cycles Int Int
    | Travels Int Int
    | Unsettled


{-| Fixed polycubes of a size per fate, each taken as a seed.
-}
type alias AutomatonCensus =
    { dying : Int
    , still : Int
    , cycling : Int
    , travelling : Int
    , unsettled : Int
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
`seedKind` and `rule`, as the backend names them.
-}
port requestAutomatonRunPort : Encode.Value -> Cmd msg


port automatonRunFinishedPort : (Decode.Value -> msg) -> Sub msg


requestAutomatonRun : { cellCount : Int, seedKind : String, rule : String } -> Cmd msg
requestAutomatonRun request =
    requestAutomatonRunPort
        (Encode.object
            [ ( "cellCount", Encode.int request.cellCount )
            , ( "seedKind", Encode.string request.seedKind )
            , ( "rule", Encode.string request.rule )
            ]
        )


automatonRunFinished : (Result String AutomatonRun -> msg) -> Sub msg
automatonRunFinished toMsg =
    automatonRunFinishedPort (decodeResult automatonRunDecoder >> toMsg)


{-| Asks for the fates of all polycubes of a size under a rule.
-}
port requestAutomatonCensusPort : Encode.Value -> Cmd msg


port automatonCensusFinishedPort : (Decode.Value -> msg) -> Sub msg


requestAutomatonCensus : { cellCount : Int, rule : String } -> Cmd msg
requestAutomatonCensus request =
    requestAutomatonCensusPort
        (Encode.object
            [ ( "cellCount", Encode.int request.cellCount )
            , ( "rule", Encode.string request.rule )
            ]
        )


automatonCensusFinished : (Result String AutomatonCensus -> msg) -> Sub msg
automatonCensusFinished toMsg =
    automatonCensusFinishedPort (decodeResult automatonCensusDecoder >> toMsg)


automatonRunDecoder : Decoder AutomatonRun
automatonRunDecoder =
    Decode.map2 AutomatonRun
        (Decode.field "generations" (Decode.list cellsDecoder))
        (Decode.field "fate" fateDecoder)


fateDecoder : Decoder Fate
fateDecoder =
    let
        generation =
            Decode.field "generation" Decode.int

        period =
            Decode.field "period" Decode.int

        ofKind kind =
            case kind of
                "dies" ->
                    Decode.map Dies generation

                "still" ->
                    Decode.map Still generation

                "cycles" ->
                    Decode.map2 Cycles generation period

                "travels" ->
                    Decode.map2 Travels generation period

                "unsettled" ->
                    Decode.succeed Unsettled

                _ ->
                    Decode.fail ("unknown fate: " ++ kind)
    in
    Decode.field "kind" Decode.string |> Decode.andThen ofKind


automatonCensusDecoder : Decoder AutomatonCensus
automatonCensusDecoder =
    Decode.map5 AutomatonCensus
        (Decode.field "dying" Decode.int)
        (Decode.field "still" Decode.int)
        (Decode.field "cycling" Decode.int)
        (Decode.field "travelling" Decode.int)
        (Decode.field "unsettled" Decode.int)

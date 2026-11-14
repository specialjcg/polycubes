port module Backend exposing
    ( ClosedFormComparison
    , DrawnPolycube
    , ExtensionSize
    , Gallery
    , EnumerationStep
    , GrownPolycube
    , GrowthComparison
    , closedFormComparisonFinished
    , drawFinished
    , enumerationTraceFinished
    , galleryFinished
    , growthComparisonFinished
    , ready
    , requestClosedFormComparison
    , requestDraw
    , requestEnumerationTrace
    , requestGallery
    , requestGrowthComparison
    )

{-| Calls to the Rust backend, relayed by `public/backend.js`.
-}

import Json.Decode as Decode exposing (Decoder)
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
    , sampleCount : Int
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
    Decode.map4 Gallery
        (Decode.field "shown" (Decode.list grownPolycubeDecoder))
        (Decode.field "uniformHistogram" (Decode.list Decode.int))
        (Decode.field "growthHistogram" (Decode.list Decode.int))
        (Decode.field "sampleCount" Decode.int)

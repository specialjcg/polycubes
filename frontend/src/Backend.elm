port module Backend exposing
    ( DrawnPolycube
    , EnumerationStep
    , GrownPolycube
    , GrowthComparison
    , drawFinished
    , enumerationTraceFinished
    , growthComparisonFinished
    , ready
    , requestDraw
    , requestEnumerationTrace
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
    let
        cellsDecoder =
            Decode.list Polycube.cellDecoder
    in
    Decode.map4 EnumerationStep
        (Decode.field "placed" cellsDecoder)
        (Decode.field "candidates" cellsDecoder)
        (Decode.field "discarded" cellsDecoder)
        (Decode.field "countsBySize" (Decode.list Decode.int))

port module Backend exposing (DrawnPolycube, drawFinished, ready, requestDraw)

{-| Calls to the Rust backend, relayed by `public/backend.js`.
-}

import Json.Decode as Decode exposing (Decoder)
import Polycube exposing (Polycube)


{-| A polycube drawn uniformly among the fixed polycubes of its size.

Rank and count are text: they may exceed what a JavaScript number holds.

-}
type alias DrawnPolycube =
    { rank : String
    , polycubeCount : String
    , cells : Polycube
    }


{-| Fires once `public/backend.js` listens to the requests. A request sent
earlier, from `init` for instance, would be lost.
-}
port ready : (() -> msg) -> Sub msg


{-| Asks for a uniform draw among the fixed polycubes of the given size.
-}
port requestDraw : Int -> Cmd msg


port drawFinishedPort : (Decode.Value -> msg) -> Sub msg


drawFinished : (Result String DrawnPolycube -> msg) -> Sub msg
drawFinished toMsg =
    drawFinishedPort (decodeDrawResult >> toMsg)


decodeDrawResult : Decode.Value -> Result String DrawnPolycube
decodeDrawResult value =
    case Decode.decodeValue drawResultDecoder value of
        Ok drawResult ->
            drawResult

        Err decodeError ->
            Err (Decode.errorToString decodeError)


drawResultDecoder : Decoder (Result String DrawnPolycube)
drawResultDecoder =
    Decode.oneOf
        [ Decode.field "drawn" drawnPolycubeDecoder |> Decode.map Ok
        , Decode.field "error" Decode.string |> Decode.map Err
        ]


drawnPolycubeDecoder : Decoder DrawnPolycube
drawnPolycubeDecoder =
    Decode.map3 DrawnPolycube
        (Decode.field "rank" Decode.string)
        (Decode.field "polycubeCount" Decode.string)
        (Decode.field "cells" (Decode.list Polycube.cellDecoder))

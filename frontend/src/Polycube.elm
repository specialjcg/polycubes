module Polycube exposing (Cell, Polycube, cellDecoder, encodeCells)

{-| A polycube as a list of unit cells, in the order they were placed.
-}

import Json.Decode as Decode exposing (Decoder)
import Json.Encode as Encode


type alias Cell =
    { x : Int
    , y : Int
    , z : Int
    }


type alias Polycube =
    List Cell


{-| The backend sends a cell as an `[x, y, z]` array.
-}
cellDecoder : Decoder Cell
cellDecoder =
    Decode.map3 Cell
        (Decode.index 0 Decode.int)
        (Decode.index 1 Decode.int)
        (Decode.index 2 Decode.int)


encodeCells : Polycube -> Encode.Value
encodeCells cells =
    Encode.list encodeCell cells


encodeCell : Cell -> Encode.Value
encodeCell cell =
    Encode.list Encode.int [ cell.x, cell.y, cell.z ]

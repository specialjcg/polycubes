module CellCountSlider exposing (initialCellCount, largestDrawnCellCount, parse, view)

{-| Choice of the number of cubes of the polycubes shown.
-}

import Html exposing (Html, input, label, text)
import Html.Attributes as Attr
import Html.Events exposing (onInput)


smallestCellCount : Int
smallestCellCount =
    1


{-| Must match `MAX_INTERACTIVE_CELL_COUNT` of the backend.
-}
largestDrawnCellCount : Int
largestDrawnCellCount =
    12


initialCellCount : Int
initialCellCount =
    10


{-| Reads the value sent by the slider; `Nothing` if it is not a number.
-}
parse : String -> Maybe Int
parse sliderValue =
    String.toInt sliderValue


{-| Slider from one cube to `largestCellCount` cubes.
-}
view : Int -> (String -> msg) -> Int -> Html msg
view largestCellCount onChosen cellCount =
    label []
        [ text ("Cubes : " ++ String.fromInt cellCount)
        , input
            [ Attr.type_ "range"
            , Attr.min (String.fromInt smallestCellCount)
            , Attr.max (String.fromInt largestCellCount)
            , Attr.value (String.fromInt cellCount)
            , onInput onChosen
            ]
            []
        ]

module Scene exposing (view, viewAmongNeighbours, viewFirstCells)

{-| The 3D view of a polycube, drawn by the `polycube-scene` custom element
(`public/polycube-scene.js`).
-}

import Html exposing (Html)
import Html.Attributes as Attr
import Json.Encode as Encode
import Polycube exposing (Cell, Polycube)


view : Polycube -> Html msg
view polycube =
    Html.node "polycube-scene" [ cellsAttribute polycube ] []


{-| Shows only the first cells of the polycube, framed as if all were there,
so that the view does not move while the polycube grows.
-}
viewFirstCells : Int -> Polycube -> Html msg
viewFirstCells shownCellCount polycube =
    Html.node "polycube-scene"
        [ cellsAttribute polycube
        , Attr.attribute "shown" (String.fromInt shownCellCount)
        ]
        []


{-| Shows a polycube with the cells waiting to extend it, those it may no
longer hold and those of an extension, framed for the largest polycube to come, so that the view
does not move from one polycube to the next.
-}
viewAmongNeighbours :
    { largestCellCount : Int, candidates : List Cell, discarded : List Cell, added : List Cell }
    -> Polycube
    -> Html msg
viewAmongNeighbours neighbours polycube =
    Html.node "polycube-scene"
        [ Attr.attribute "extent" (String.fromInt neighbours.largestCellCount)
        , cellsAttribute polycube
        , encodedCellsAttribute "candidates" neighbours.candidates
        , encodedCellsAttribute "discarded" neighbours.discarded
        , encodedCellsAttribute "added" neighbours.added
        ]
        []


cellsAttribute : Polycube -> Html.Attribute msg
cellsAttribute polycube =
    encodedCellsAttribute "cells" polycube


encodedCellsAttribute : String -> List Cell -> Html.Attribute msg
encodedCellsAttribute name cells =
    Attr.attribute name (Encode.encode 0 (Polycube.encodeCells cells))

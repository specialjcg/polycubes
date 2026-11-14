module Scene exposing (view, viewAmongNeighbours, viewFirstCells, viewFramed)

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


{-| Shows cells in a view framed once on the box between two opposite
corner cells, so that it stays still while the cells spread.
-}
viewFramed : { lowestCorner : Cell, highestCorner : Cell } -> List Cell -> Html msg
viewFramed frame cells =
    Html.node "polycube-scene"
        [ encodedCellsAttribute "framed" [ frame.lowestCorner, frame.highestCorner ]
        , cellsAttribute cells
        ]
        []


cellsAttribute : Polycube -> Html.Attribute msg
cellsAttribute polycube =
    encodedCellsAttribute "cells" polycube


encodedCellsAttribute : String -> List Cell -> Html.Attribute msg
encodedCellsAttribute name cells =
    Attr.attribute name (Encode.encode 0 (Polycube.encodeCells cells))

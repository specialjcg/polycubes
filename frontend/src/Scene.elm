module Scene exposing (view, viewFirstCells)

{-| The 3D view of a polycube, drawn by the `polycube-scene` custom element
(`public/polycube-scene.js`).
-}

import Html exposing (Html)
import Html.Attributes as Attr
import Json.Encode as Encode
import Polycube exposing (Polycube)


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


cellsAttribute : Polycube -> Html.Attribute msg
cellsAttribute polycube =
    Attr.attribute "cells" (Encode.encode 0 (Polycube.encodeCells polycube))

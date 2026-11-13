module Scene exposing (view)

{-| The 3D view of a polycube, drawn by the `polycube-scene` custom element
(`public/polycube-scene.js`).
-}

import Html exposing (Html)
import Html.Attributes as Attr
import Json.Encode as Encode
import Polycube exposing (Polycube)


view : Polycube -> Html msg
view polycube =
    Html.node "polycube-scene"
        [ Attr.attribute "cells" (Encode.encode 0 (Polycube.encodeCells polycube)) ]
        []

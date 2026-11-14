module Main exposing (main)

{-| Animated views on polycubes. Each page is a module of `Page`, holding its
own state, messages and view.
-}

import Backend
import Browser
import Html exposing (Html, button, h1, main_, nav, text)
import Html.Attributes as Attr
import Html.Events exposing (onClick)
import Page.ClosedForm
import Page.Draw
import Page.Enumeration
import Page.Growth


type Page
    = GrowthPage
    | EnumerationPage
    | ClosedFormPage
    | DrawPage


type alias Model =
    { shownPage : Page
    , growth : Page.Growth.Model
    , enumeration : Page.Enumeration.Model
    , closedForm : Page.ClosedForm.Model
    , draw : Page.Draw.Model
    }


type Msg
    = BackendReady
    | PageChosen Page
    | GrowthMsg Page.Growth.Msg
    | EnumerationMsg Page.Enumeration.Msg
    | ClosedFormMsg Page.ClosedForm.Msg
    | DrawMsg Page.Draw.Msg


main : Program () Model Msg
main =
    Browser.element
        { init = \_ -> ( init, Cmd.none )
        , update = update
        , view = view
        , subscriptions = subscriptions
        }


init : Model
init =
    { shownPage = GrowthPage
    , growth = Page.Growth.init
    , enumeration = Page.Enumeration.init
    , closedForm = Page.ClosedForm.init
    , draw = Page.Draw.init
    }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        BackendReady ->
            let
                ( growth, growthCmd ) =
                    Page.Growth.start model.growth

                ( enumeration, enumerationCmd ) =
                    Page.Enumeration.start model.enumeration

                ( closedForm, closedFormCmd ) =
                    Page.ClosedForm.start model.closedForm

                ( draw, drawCmd ) =
                    Page.Draw.start model.draw
            in
            ( { model | growth = growth, enumeration = enumeration, closedForm = closedForm, draw = draw }
            , Cmd.batch
                [ Cmd.map GrowthMsg growthCmd
                , Cmd.map EnumerationMsg enumerationCmd
                , Cmd.map ClosedFormMsg closedFormCmd
                , Cmd.map DrawMsg drawCmd
                ]
            )

        PageChosen page ->
            ( { model | shownPage = page }, Cmd.none )

        GrowthMsg growthMsg ->
            let
                ( growth, growthCmd ) =
                    Page.Growth.update growthMsg model.growth
            in
            ( { model | growth = growth }, Cmd.map GrowthMsg growthCmd )

        EnumerationMsg enumerationMsg ->
            let
                ( enumeration, enumerationCmd ) =
                    Page.Enumeration.update enumerationMsg model.enumeration
            in
            ( { model | enumeration = enumeration }, Cmd.map EnumerationMsg enumerationCmd )

        ClosedFormMsg closedFormMsg ->
            let
                ( closedForm, closedFormCmd ) =
                    Page.ClosedForm.update closedFormMsg model.closedForm
            in
            ( { model | closedForm = closedForm }, Cmd.map ClosedFormMsg closedFormCmd )

        DrawMsg drawMsg ->
            let
                ( draw, drawCmd ) =
                    Page.Draw.update drawMsg model.draw
            in
            ( { model | draw = draw }, Cmd.map DrawMsg drawCmd )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Backend.ready (\_ -> BackendReady)
        , Sub.map GrowthMsg (Page.Growth.subscriptions model.growth)
        , Sub.map EnumerationMsg (Page.Enumeration.subscriptions model.enumeration)
        , Sub.map ClosedFormMsg (Page.ClosedForm.subscriptions model.closedForm)
        , Sub.map DrawMsg Page.Draw.subscriptions
        ]


view : Model -> Html Msg
view model =
    main_ []
        [ h1 [] [ text "Polycubes" ]
        , nav []
            [ viewPageButton model.shownPage GrowthPage "Croissance biaisée contre tirage uniforme"
            , viewPageButton model.shownPage EnumerationPage "Énumération de Redelmeier"
            , viewPageButton model.shownPage ClosedFormPage "Formule fermée"
            , viewPageButton model.shownPage DrawPage "Tirage uniforme"
            ]
        , case model.shownPage of
            GrowthPage ->
                Html.map GrowthMsg (Page.Growth.view model.growth)

            EnumerationPage ->
                Html.map EnumerationMsg (Page.Enumeration.view model.enumeration)

            ClosedFormPage ->
                Html.map ClosedFormMsg (Page.ClosedForm.view model.closedForm)

            DrawPage ->
                Html.map DrawMsg (Page.Draw.view model.draw)
        ]


viewPageButton : Page -> Page -> String -> Html Msg
viewPageButton shownPage page title =
    button
        [ onClick (PageChosen page)
        , Attr.classList [ ( "tab", True ), ( "shown", page == shownPage ) ]
        ]
        [ text title ]

import {css, styled} from "@mui/material";
import {useTranslation, Trans} from "react-i18next";

import {Header, Sidebar} from "@widgets/sidebar";
import {CommonTemplate} from "@shared/ui/templates";
import {Layout} from "@shared/lib/layout";
import {Text} from "@shared/ui/atoms";

export const AboutPage: React.FC = () => {
  const {t} = useTranslation("about");

  return (
    <>
      <Sidebar.Navigational />
      <Sidebar.Social />

      <CommonTemplate>
        <Header>{t("header")}</Header>

        <Layout.Col w="100%" gap={5}>
          <Layout.Col w="50%" gap={2}>
            <Description>
              <Trans
                ns="about"
                i18nKey="description1"
                components={[
                  <Bold key="ekittens-bold" />,
                  <NotEssential key="exploding-kitten-not-essential" />,
                ]}
              />
            </Description>

            <Description2>{t("description2")}</Description2>
          </Layout.Col>
        </Layout.Col>
      </CommonTemplate>
    </>
  );
};


interface DescriptionStyledProps {
  bold?: boolean;
}

const Description = styled(Text)<DescriptionStyledProps>`
  font-weight: 400;
  font-size: 1.6rem;
  line-height: 1.5;

  ${({bold}) =>
    bold &&
    css`
      font-weight: 700;
    `}
`;

const Description2 = styled(Description)`
  color: ${({theme}) => theme.palette.text.secondary};
  white-space: break-spaces;
`;

const Bold = styled("p")`
  font-weight: 700;
  display: inline;
  font-style: italic;
`;

const NotEssential = styled("p")`
  color: ${({theme}) => theme.palette.text.secondary};
  font-weight: 400;
  display: inline;
`;
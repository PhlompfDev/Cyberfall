// Tooltip text, used from r6/tweaks/Cyberfall/cyberfall.yaml.
module Cyberfall.Localization
import Codeware.Localization.*

public class CyberfallLocalizationProvider extends ModLocalizationProvider {
  public func GetPackage(language: CName) -> ref<ModLocalizationPackage> {
    switch language {
      case n"en-us": return new CyberfallEnglish();
      default: return null;
    }
  }

  public func GetFallback() -> CName {
    return n"en-us";
  }
}

public class CyberfallEnglish extends ModLocalizationPackage {
  protected func DefineTexts() -> Void {
    this.Text("Cyberfall-Tendons-FallSpeed",
      "Removes falling penalty from <Rich color=\"TooltipText.cyberwareDescriptionHighlightColor\" style=\"Semi-Bold\">double jump</> and <Rich color=\"TooltipText.cyberwareDescriptionHighlightColor\" style=\"Semi-Bold\">dash</>.");
    this.Text("Cyberfall-Tendons-HardLanding",
      "Rough landings don't slow you down or deal <Rich color=\"TooltipText.cyberwareDescriptionHighlightColor\" style=\"Semi-Bold\">fall damage</>.");
    this.Text("Cyberfall-Ankles-ChargedKick",
      "Hold <Rich color=\"TooltipText.cyberwareDescriptionHighlightColor\" style=\"Semi-Bold\">Jump</> while wallrunning for a <Rich color=\"TooltipText.cyberwareDescriptionHighlightColor\" style=\"Semi-Bold\">charged wall kick</>.");
  }

  protected func DefineSubtitles() -> Void {}
}

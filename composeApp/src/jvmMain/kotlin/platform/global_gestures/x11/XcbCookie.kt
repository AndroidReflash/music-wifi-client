package platform.global_gestures.x11

import com.sun.jna.Structure

/**
 * Квитанция запроса XCB: номер запроса, по которому потом забирают ответ или ошибку. В C у каждого
 * запроса свой тип квитанции, но все они — структура из одного `unsigned int`, передаваемая по
 * значению; JNA имена типов не важны, поэтому класс один.
 */
@Structure.FieldOrder("sequence")
open class XcbCookie : Structure() {

    @JvmField
    var sequence: Int = 0

    class ByValue : XcbCookie(), Structure.ByValue
}
